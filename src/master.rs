use i2c_common::{
    msg::{I2cMsg,I2cMsgFlags},
    I2cFuncFlags,
    I2cSpeedMode
};
use osl::{error::Result, vec::Vec};
use tock_registers::LocalRegisterCopy;

use crate::{
    common::{DwI2cSclLHCnt,DwI2cCmdErr}, 
    registers::*, 
    I2cDwCoreDriver, 
    I2cDwDriverConfig
};

/// The I2cDesignware Driver
#[allow(dead_code)]
pub struct I2cDwMasterDriver<'a> {
    /// I2c Config  register set value
    cfg: LocalRegisterCopy<u32, IC_CON::Register>,
    /// core Driver
    driver: I2cDwCoreDriver,
    /// I2c scl_LHCNT
    lhcnt: DwI2cSclLHCnt,
    /// Fifo
    tx_fifo_depth: u32,
    rx_fifo_depth: u32,

    /// XferData
    msgs:Vec<I2cMsg<'a>>,
    /// run time hadware error code
    cmd_err: DwI2cCmdErr,
    /// the element index of the current rx message in the msgs array
    msg_read_idx: u32,
    /// the buf index of the current msg[msg_read_idx] buf  
    rx_buf_index: u32,
    /// the element index of the current tx message in the msgs array
    msg_write_idx: u32,
    /// error status of the current transfer
    msg_err: u32,
    /// copy of the TX_ABRT_SOURCE register
    abort_source: u32,
    /// current master-rx elements in tx fifo
    rx_outstanding: u32,
}

unsafe impl Sync for I2cDwMasterDriver<'_> {}
unsafe impl Send for I2cDwMasterDriver<'_> {}

impl <'a> I2cDwMasterDriver<'a> {

    /// Create a new I2cDesignwarDriver
    pub fn new(config: I2cDwDriverConfig, base_addr: *mut u8) -> I2cDwMasterDriver<'a> {
        Self {
            cfg: LocalRegisterCopy::new(0),
            driver: I2cDwCoreDriver::new(config, base_addr),
            lhcnt: DwI2cSclLHCnt::default(),
            tx_fifo_depth: 0,
            rx_fifo_depth: 0,

            msgs: Vec::new(),
            cmd_err: DwI2cCmdErr::from_bits(0),
            msg_read_idx: 0,
            msg_write_idx: 0,
            msg_err: 0,
            abort_source: 0,
            rx_outstanding: 0,
        }
    }

    fn reinit_xfer(&mut self, msgs: Vec<I2cMsg<'a>>) {
        self.msgs =  msgs;
        self.cmd_err = DwI2cCmdErr::from_bits(0);
        self.msg_write_idx = 0;
        self.msg_err = 0;
        self.abort_source =  0;
        self.rx_outstanding= 0;
    }

    fn is_enable_10bitaddr(&self) -> bool {
        self.msgs[self.msg_write_idx as usize].flags().contains(I2cMsgFlags::I2cMasterTen)
    }

    /// Interrupt service routine. This gets called whenever an I2C master interrupt
    /// occurs
    pub fn irq_handler(&mut self) {
        let enable = self.driver.ic_enable();
        let stat = self.driver.ic_raw_intr_stat().get();
        // check raw intr stat
        if !enable.is_set(IC_ENABLE::ENABLE) &&
            (stat & !0b100000000) == 0  {
                return 0;
        }

        let stat = self.driver.read_and_clean_intrbits(&mut self.abort_source, self.rx_outstanding);

        if !self.driver.is_active() {
            /// Unexpected interrupt in driver point of view. State
            /// variables are either unset or stale so acknowledge and
            /// disable interrupts for suppressing further interrupts if
            /// interrupt really came from this HW (E.g. firmware has left
            /// the HW active).
            self.driver.disable_all_interrupt();
            return 0;
        }

        if stat.is_set(IC_INTR::TX_ABRT) {
            self.xfer_data.cmd_err |= DwI2cCmdErr::TX_ABRT;
            self.xfer_data.rx_outstanding = 0;
            // Anytime TX_ABRT is set, the contents of the tx/rx
            // buffers are flushed. Make sure to skip them.
            self.driver.disable_all_interrupt();

            // TOOD: finish complete
            return 0;
        }
        
        if stat.is_set(IC_INTR::RX_FULL) {
            self.xfer_read_msgs();
        }
               

    }

    /// Initialize the designware I2C driver config
    pub fn setup(&mut self) -> Result<()> {
        // com and speed check must be the first step
        self.driver.com_type_check()?;
        self.driver.speed_check()?;

        // init config
        self.config_init()?;
        self.scl_lhcnt_init()?;
        self.driver.sda_hold_time_init()?;
        self.fifo_size_init();

        // Initialize the designware I2C master hardware
        self.master_setup()?;

        Ok(())
    }

    /// return  i2c functionality
    pub fn get_functionality(&self) -> I2cFuncFlags {
        self.driver.functionality
    }

    /// Prepare controller for a transaction and call xfer_msg
    pub fn master_xfer(&mut self, msgs: Vec<I2cMsg<'a>>) -> Result<()> {
        self.reinit_xfer(msgs);
        // wait bus free
        self.driver.wait_bus_not_busy()?;
        self.xfer_init();

        // wait xfer complete
        
        Ok(())
    }

    fn xfer_init(&mut self) {
        // deisable the adapter
        self.driver.disable_controler();
        let mut ic_tar: LocalRegisterCopy<u32, TAR::Register> =  LocalRegisterCopy::new(0);
        // If the slave address is ten bit address, enable 10BITADDR
        if self.is_enable_10bitaddr() {
            self.driver.enable_10bitaddr(true);
        } else {
            ic_tar.modify(IC_TAR::IC_10BITADDR_MASTER.val(0b1));
            self.driver.enable_10bitaddr(false);
        }
        
        ic_tar.modify(IC_TAR::TAR.val(
                self.msgs[self.msg_write_idx as usize].addr()
            ));
        self.driver.set_ic_tar(ic_tar.get());
        
        // Enforce disabled interrupts (due to HW issues)
        self.dirver.disable_all_interrupt();

        // Enable the adapter
        self.dirver.enable_controler();

        // Dummy read to avoid the register getting stuck on Bay Trail
        let _ = self.driver.ic_enable_status();    
        // Clear and enable interrupts
        self.driver.clear_all_interrupt();
        let mut mask = LocalRegisterCopy::new(0).modify(IC_INTR::RX_FULL.val(0b1));
        mask.modify(IC_INTR::TX_ABRT.val(0b1));
        mask.modify(IC_INTR::STOP_DET.val(0b1));
        mask.modify(IC_INTR::TX_EMPTY.val(0b1));
        self.driver.set_interrupt_mask(&mask);
    }

    fn xfer_read_msgs(&mut self) {
        let msgs = &mut self.msgs[self.msg_read_idx];
        for m in &mut msgs {
            if !m.flags().contains(I2cMsgFlags::I2cMasterRead) {
                self.msg_read_idx +=1;
                continue
            }

            let mut rx_valid = self.driver.ic_rxflr().get();
            
            for i in 0..rx_valid {
                let ic_data_cmd = self.driver.ic_data_cmd().read(IC_DATA_CMD::DATA);
                
            }
        }
    }


    /// functionality and cfg init
    fn config_init(&mut self) -> Result<()> {
        // init functionality
        let functionality = I2cFuncFlags::TEN_BIT_ADDR;
        self.driver.functionality_init(functionality);

        // init master cfg
        self.cfg.modify(IC_CON::MASTER_MODE.val(1));
        self.cfg.modify(IC_CON::IC_SLAVE_DISABLE.val(1));
        self.cfg.modify(IC_CON::IC_RESTART_EN.val(1));

        // On AMD platforms BIOS advertises the bus clear feature
        // and enables the SCL/SDA stuck low. SMU FW does the
        // bus recovery process. Driver should not ignore this BIOS
        // advertisement of bus clear feature.
        if self.driver.ic_con().is_set(IC_CON::BUS_CLEAR_FEATURE_CTRL)
        {
            self.cfg.modify(IC_CON::BUS_CLEAR_FEATURE_CTRL.val(1));
        }

        self.driver.cfg_init_speed(&mut self.cfg);
        Ok(())
    }

    fn master_setup(&mut self) -> Result<()> {
        // Disable the adapter
        self.driver.disable_controler();

        // Write standard speed timing parameters
        self.driver.set_lhcnt(&self.lhcnt);
        // Write SDA hold time if supported
        self.driver.write_sda_hold_time();
        // Write fifo
        self.driver.set_fifo(self.tx_fifo_depth / 2, 0);

        // set IC_CON
        self.driver.set_ic_con(&self.cfg);
        Ok(())
    }

    fn fifo_size_init(&mut self) {
        let com_param_1 = self.driver.ic_comp_param_1();
        self.tx_fifo_depth = com_param_1.read(IC_COMP_PARAM_1::TX_BUFFER_DEPTH) + 1;
        self.rx_fifo_depth = com_param_1.read(IC_COMP_PARAM_1::RX_BUFFER_DEPTH) + 1;
        log_info!(
            "I2C fifo_depth RX:TX = {}: {}",
            self.rx_fifo_depth,
            self.tx_fifo_depth
        );
    }

    fn scl_lhcnt_init(&mut self) -> Result<()> {
        let driver = &mut self.driver;
        let ic_clk = driver.ext_config.clk_rate_khz;
        let mut scl_fall_ns = driver.ext_config.timing.get_scl_fall_ns();
        let mut sda_fall_ns = driver.ext_config.timing.get_sda_fall_ns();

        // Set standard and fast speed dividers for high/low periods
        if scl_fall_ns == 0 {
            scl_fall_ns = 300;
        }

        if sda_fall_ns == 0 {
            sda_fall_ns = 300;
        }

        // tLOW = 4.7 us and no offset
        self.lhcnt.ss_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 4700, scl_fall_ns, 0) as u16;
        // tHigh = 4 us and no offset DW default
        self.lhcnt.ss_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 4000, sda_fall_ns, false, 0) as u16;
        log_info!(
            "I2C dw Standard Mode HCNT:LCNT = {} : {}",
            self.lhcnt.ss_hcnt,
            self.lhcnt.ss_lcnt
        );

        let speed_mode = driver.speed_mode;
        if speed_mode == I2cSpeedMode::FastPlusMode {
            self.lhcnt.fs_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 500, scl_fall_ns, 0) as u16;
            self.lhcnt.fs_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 260, sda_fall_ns, false, 0) as u16;
            log_info!(
                "I2C Fast Plus Mode HCNT:LCNT = {} : {}",
                self.lhcnt.ss_hcnt,
                self.lhcnt.ss_lcnt
            );
        } else {
            self.lhcnt.fs_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 1300, scl_fall_ns, 0) as u16;
            self.lhcnt.fs_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 600, sda_fall_ns, false, 0) as u16;
            log_info!(
                "I2C Fast Mode HCNT:LCNT = {} : {}",
                self.lhcnt.fs_hcnt,
                self.lhcnt.fs_lcnt
            );
        }

        if speed_mode == I2cSpeedMode::HighSpeedMode {
            self.lhcnt.hs_lcnt = DwI2cSclLHCnt::scl_lcnt(ic_clk, 320, scl_fall_ns, 0) as u16;
            self.lhcnt.hs_hcnt = DwI2cSclLHCnt::scl_hcnt(ic_clk, 160, sda_fall_ns, false, 0) as u16;
            log_info!(
                "I2C High Speed Mode HCNT:LCNT = {} : {}",
                self.lhcnt.hs_hcnt,
                self.lhcnt.hs_lcnt
            );
        }

        Ok(())
    }
}
