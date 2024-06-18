use osl::{
    error::{to_error, Errno, Result},
    vec::Vec,
    sync::{OslCompletion,GeneralComplete, SpinLock, new_spinlock, Arc},
    driver::irq,
    driver::irq::{to_irq_return, ReturnEnum},
    driver::i2c::{I2cMsg, I2cMsgFlags, I2cFuncFlags, I2cSpeedMode, I2C_SMBUS_BLOCK_MAX, GeneralI2cMsg},
};

#[allow(unused_imports)]
use tock_registers::{
    LocalRegisterCopy,
    fields::FieldValue,
};

#[allow(unused_imports)]
use crate::{
    common::{DwI2cCmdErr, DwI2cSclLHCnt, DwI2cStatus},
    registers::*,
    I2cDwCoreDriver, I2cDwDriverConfig,
};

enum TransferResult  {
    // Unexpected irq
    UnExpectedInterrupt,
    // Recive IRQ abort
    Abort,
    // All msgs are process success
    Fininsh,
    // Still need next irq
    Continue,
}

/// Master driver transfer abstract
#[allow(dead_code)]
struct MasterXfer {
    /// XferData
    msgs: Vec<I2cMsg>,
    /// run time hadware error code
    cmd_err: DwI2cCmdErr,
    /// the element index of the current rx message in the msgs array
    msg_read_idx: usize,
    /// the element index of the current tx message in the msgs array
    msg_write_idx: usize,
    /// error status of the current transfer
    msg_err: Result<()>,
    /// copy of the TX_ABRT_SOURCE register
    abort_source: LocalRegisterCopy<u32, IC_TX_ABRT_SOURCE::Register>,
    /// current master-rx elements in tx fifo
    rx_outstanding: isize,
    /// Driver Status
    status: DwI2cStatus,
}

impl Default for MasterXfer {
    /// Create an empty XferData
    fn default() -> Self {
        Self {
            msgs: Vec::new(),
            cmd_err: DwI2cCmdErr::from_bits(0).unwrap(),
            msg_read_idx: 0,
            msg_write_idx: 0,
            msg_err: Ok(()),
            abort_source: LocalRegisterCopy::new(0),
            rx_outstanding: 0,
            status: DwI2cStatus::empty(),
        }
    }
}

impl MasterXfer {
    #[allow(dead_code)]
    fn init(&mut self, msgs: Vec<I2cMsg>) {
        self.msgs = msgs;
        self.cmd_err = DwI2cCmdErr::from_bits(0).unwrap();
        self.msg_read_idx = 0;
        self.msg_write_idx = 0;
        self.msg_err = Ok(());
        self.abort_source = LocalRegisterCopy::new(0);
        self.rx_outstanding = 0;
        self.status = DwI2cStatus::empty();
    }

    #[inline]
    pub(crate) fn is_empty_status(&self) -> bool {
        self.status.is_empty()
    }

    #[inline]
    pub(crate) fn clear_active(&mut self) {
        self.status &= !DwI2cStatus::ACTIVE;
    }

    #[inline]
    pub(crate) fn set_active(&mut self) {
        self.status |= DwI2cStatus::ACTIVE;
    }

    #[inline]
    pub(crate) fn is_active(&self) -> bool {
        self.status.contains(DwI2cStatus::ACTIVE)
    }

    #[inline]
    pub(crate) fn is_write_in_progress(&self) -> bool {
        self.status.contains(DwI2cStatus::WriteInProgress)
    }

    #[inline]
    pub(crate) fn set_write_in_progress(&mut self, set: bool) {
        if set {
            self.status |= DwI2cStatus::WriteInProgress;
        } else {
            self.status &= !DwI2cStatus::WriteInProgress;
        }
    }

    fn prepare(&mut self, msgs: Vec<I2cMsg>, master_driver: &I2cDwMasterDriver) {
        self.init(msgs);
        let core_driver = &master_driver.driver;
        // disable the adapter
        master_driver.disable(false);

        let first_msg = &self.msgs[self.msg_write_idx as usize];
        let mut ic_tar: LocalRegisterCopy<u32, IC_TAR::Register> = LocalRegisterCopy::new(0);
        if first_msg.flags().contains(I2cMsgFlags::I2cAddrTen){
            core_driver.enable_10bitaddr(true);
        } else {
            ic_tar.modify(IC_TAR::IC_10BITADDR_MASTER.val(0b1));
            core_driver.enable_10bitaddr(false);
        }

        ic_tar.modify(IC_TAR::TAR.val(first_msg.addr().into()));
        core_driver.write_ic_tar(&ic_tar);

        // Enforce disabled interrupts (due to HW issues) 
        core_driver.disable_all_interrupt();

        // Enable the adapter
        core_driver.enable_controler();
        self.set_active();
        // Dummy read to avoid the register getting stuck on Bay Trail
        let _ = core_driver.ic_enable_status();
    }

    fn irq_process(&mut self, master_driver: &I2cDwMasterDriver) -> TransferResult {
        let core_driver = &master_driver.driver;
        let (stat, abort_source) = 
            core_driver.read_and_clean_intrbits(self.rx_outstanding);
        self.abort_source = abort_source;

        // Unexpected interrupt in driver point of view. State
        // variables are either unset or stale so acknowledge and
        // disable interrupts for suppressing further interrupts if
        // interrupt really came from this HW (E.g. firmware has left
        // the HW active).
        assert!(self.is_active());
        if !self.is_active() {
            return TransferResult::UnExpectedInterrupt; 
        }

        if stat.is_set(IC_INTR::TX_ABRT) {
            self.cmd_err |= DwI2cCmdErr::TX_ABRT;
            self.status = DwI2cStatus::empty();
            log_err!("recieve abort irq");
            return TransferResult::Abort; 
        }

        if stat.is_set(IC_INTR::RX_FULL) {
            self.read_msgs(&master_driver);
        }

        if stat.is_set(IC_INTR::TX_EMPTY) {
            self.write_msgs(&master_driver);
        }

        if  (stat.is_set(IC_INTR::STOP_DET) || self.msg_err.is_err()) 
            && self.rx_outstanding == 0 {
                return TransferResult::Fininsh;
        }

        return TransferResult::Continue;
    }

    fn exit(&mut self, master_driver: &I2cDwMasterDriver) -> Result<()> {
        // We must disable the adapter before returning and signaling the end
        // of the current transfer. Otherwise the hardware might continue
        // generating interrupts which in turn causes a race condition with
        // the following transfer.  Needs some more investigation if the
        // additional interrupts are a hardware bug or this driver doesn't
        // handle them correctly yet.
        master_driver.disable(true);
        self.clear_active();

        match self.msg_err {
            Err(e) => {
                log_err!("i2c dw transfer process msg error: {:?}",e);
                return Err(e);
            }
            Ok(_) => {},
        }

        match self.cmd_err {
            DwI2cCmdErr::TX_ABRT => {
                log_err!("i2c dw transfer recv tx_abort");
                self.handle_tx_abort()?;
            }
            _ => {},
        }

        if !self.is_empty_status() {
            log_err!("transfer terminated early - interrupt latency too high?");
            return to_error(Errno::Io);
        }
        Ok(())
    }


    fn handle_tx_abort(&mut self) -> Result<()> {
        let abort_source = self.abort_source;
        if abort_source.matches_any(&self.tx_abort_noack()){
            return to_error(Errno::Io);
        }
        if abort_source.is_set(IC_TX_ABRT_SOURCE::ARB_LOST){
            return to_error(Errno::Again);
        } else if abort_source.is_set(IC_TX_ABRT_SOURCE::ABRT_GCALL_READ){
            return to_error(Errno::InvalidArgs);
        } else {
            return to_error(Errno::Io);
        }
    }

    fn write_msgs(&mut self, master_driver: &I2cDwMasterDriver) {
        let msg_len = self.msgs.len();
        let core_driver = &master_driver.driver;
        
        let mut intr_mask = I2cDwMasterDriver::master_default_intr_mask();
        let addr = self.msgs[self.msg_write_idx].addr();
        let mut need_restart = false;
        loop {
            let write_idx = self.msg_write_idx;
            if write_idx >= msg_len {
                break;
            }

            if !self.is_write_in_progress() {
                //If both IC_EMPTYFIFO_HOLD_MASTER_EN and
                //IC_RESTART_EN are set, we must manually
                //set restart bit between messages.
                if master_driver.cfg.is_set(IC_CON::IC_RESTART_EN) && 
                    write_idx > 0
                {
                    need_restart = true;           
                }
            }

            let msg = &mut self.msgs[write_idx];

            if msg.addr() != addr {
                self.msg_err = to_error(Errno::InvalidArgs);
                break;
            }

            let flr = core_driver.ic_txflr().get();
            let mut tx_limit = master_driver.tx_fifo_depth - flr;
                
            let flr = core_driver.ic_rxflr().get();
            let mut rx_limit = master_driver.rx_fifo_depth - flr;
            
            loop {
                if msg.send_end() || rx_limit <=0 || tx_limit <=0 {
                    break;
                }
                let mut cmd: LocalRegisterCopy<u32, IC_DATA_CMD::Register> = LocalRegisterCopy::new(0);
                // If IC_EMPTYFIFO_HOLD_MASTER_EN is set we must
                // manually set the stop bit. However, it cannot be
                // detected from the registers so we set it always
                // when writing/reading the last byte.
                //
                // i2c-core always sets the buffer length of
                // I2C_FUNC_SMBUS_BLOCK_DATA to 1. The length will
                // be adjusted when receiving the first byte.
                // Thus we can't stop the transaction here.
                if write_idx == msg_len-1 &&
                    !msg.flags().contains(I2cMsgFlags::I2cMasterRecvLen) &&
                    msg.send_left_last() {
                    cmd.modify(IC_DATA_CMD::STOP.val(0b1));
                }

                if need_restart {
                    cmd.modify(IC_DATA_CMD::RESTART.val(0b1));
                    need_restart = false;
                }

                if msg.flags().contains(I2cMsgFlags::I2cMasterRead) {
                    /* Avoid rx buffer overrun */
                    if self.rx_outstanding >=
                         master_driver.rx_fifo_depth.try_into().unwrap() {
                        break;
                    }
                    cmd.modify(IC_DATA_CMD::CMD.val(0b1));
                    rx_limit -= 1;
                    self.rx_outstanding += 1;
                    msg.inc_recieve_cmd_cnt();
                } else {
                    cmd.modify(IC_DATA_CMD::DAT.val(msg.pop_front_byte() as u32)); 
                }
                core_driver.write_ic_data_cmd(&cmd);
                tx_limit -=1;
            }

            // Because we don't know the buffer length in the
            // I2C_FUNC_SMBUS_BLOCK_DATA case, we can't stop the
            // transaction here. Also disable the TX_EMPTY IRQ
            // while waiting for the data length byte to avoid the
            // bogus interrupts flood.
            if msg.flags().contains(I2cMsgFlags::I2cMasterRecvLen) {
                self.set_write_in_progress(true);
                intr_mask.modify(IC_INTR::TX_EMPTY.val(0b0));
                break;
            } else if !msg.send_end() {
                // wait next time TX_EMPTY interrupt
                self.set_write_in_progress(true);
                break;
            } else {
                self.set_write_in_progress(false);
                self.msg_write_idx +=1;
            }
        }
        
        // If i2c_msg index search is completed, we don't need TX_EMPTY
        // interrupt any more.
        if self.msg_write_idx == msg_len {
            intr_mask.modify(IC_INTR::TX_EMPTY.val(0b0));
        }

        if self.msg_err.is_err() {
            intr_mask = LocalRegisterCopy::new(0);
        }

        core_driver.write_interrupt_mask(&intr_mask);
    }

    fn read_msgs(&mut self, master_driver: &I2cDwMasterDriver) {
        let msg_len = self.msgs.len();
        let core_driver = &master_driver.driver;

        loop {
            let read_idx = self.msg_read_idx;
            if read_idx >= msg_len {
                break;
            }

            let msg = &mut(self.msgs[read_idx]);

            if !msg.flags().contains(I2cMsgFlags::I2cMasterRead) {
                self.msg_read_idx += 1;
                continue
            }

            let rx_valid = core_driver.ic_rxflr().get();
            for _ in 0..rx_valid {
                // check if buf can be write
                if msg.recieve_end() {
                    break;
                }

                let mut ic_data = core_driver.ic_data_cmd().read(IC_DATA_CMD::DAT) as u8;
                // Ensure length byte is a valid value
                if msg.flags().contains(I2cMsgFlags::I2cMasterRecvLen) {
                    // if IC_EMPTYFIFO_HOLD_MASTER_EN is set, which cannot be
                    // detected from the registers, the controller can be
                    // disabled if the STOP bit is set. But it is only set
                    // after receiving block data response length in
                    // I2C_FUNC_SMBUS_BLOCK_DATA case. That needs to read
                    // another byte with STOP bit set when the block data
                    // response length is invalid to complete the transaction.
                    if ic_data == 0 || ic_data > I2C_SMBUS_BLOCK_MAX {
                        ic_data = 1;
                    }
                    let mut buf_len = ic_data as usize;
                    // Adjust the buffer length and mask the flag 
                    // after receiving the first byte.
                    if msg.flags().contains(I2cMsgFlags::I2cClientPec) {
                        buf_len+=2;
                    } else {
                        buf_len+=1;
                    };
                    msg.modify_recieve_threshold(buf_len);
                    // cacluate read_cmd_cnt
                    msg.modify_recieve_cmd_cnt(self.rx_outstanding.min(buf_len as isize));
                    msg.remove_flag(I2cMsgFlags::I2cMasterRecvLen);
                    
                    // Received buffer length, re-enable TX_EMPTY interrupt
                    // to resume the SMBUS transaction.
                    core_driver.enable_tx_empty_intr(true);
                }
                msg.push_byte(ic_data.try_into().unwrap());
                self.rx_outstanding -= 1;
            }
            
            if !msg.recieve_end() {
                // wait next time RX_FULL interrupt
                return
            } else {
                self.msg_read_idx +=1;
            }
        }
    }

    fn tx_abort_noack(&self) -> [FieldValue<u32,IC_TX_ABRT_SOURCE::Register>;5] {
        [
            IC_TX_ABRT_SOURCE::ABRT_7B_ADDR_NOACK.val(0b1),
            IC_TX_ABRT_SOURCE::ABRT_10ADDR1_NOACK.val(0b1),
            IC_TX_ABRT_SOURCE::ABRT_10ADDR2_NOACK.val(0b1),
            IC_TX_ABRT_SOURCE::ABRT_TXDATA_NOACK.val(0b1),
            IC_TX_ABRT_SOURCE::ABRT_GCALL_NOACK.val(0b1)
        ]
    }

}

/// The I2cDesignware Driver
#[allow(dead_code)]
pub struct I2cDwMasterDriver {
    /// I2c Config  register set value
    cfg: LocalRegisterCopy<u32, IC_CON::Register>,
    /// core Driver
    driver: I2cDwCoreDriver,
    /// I2c scl_LHCNT
    lhcnt: DwI2cSclLHCnt,
    /// Fifo
    tx_fifo_depth: u32,
    rx_fifo_depth: u32,
    
    /// Arc completion 
    cmd_complete: Arc<OslCompletion>,

    /// Since xfer will be used in interrupt handler,
    /// the data needs a concurrent mechanism to ensure safety. 
    /// The driver will ensure that it will not be triggered
    /// by interrupts when using locks,
    /// so there is no need to use spin_noirq
    #[cfg(feature = "linux")]
    xfer: Arc<SpinLock<MasterXfer>>,
    #[cfg(feature = "arceos")]
    xfer: SpinLock<MasterXfer>,
}

impl I2cDwMasterDriver {
    /// Create a new I2cDesignwarDriver
    pub fn new(config: I2cDwDriverConfig, base_addr: *mut u8) -> Self {
        Self {
            cfg: LocalRegisterCopy::new(0),
            driver: I2cDwCoreDriver::new(config, base_addr),
            lhcnt: DwI2cSclLHCnt::default(),
            tx_fifo_depth: 0,
            rx_fifo_depth: 0,
            cmd_complete: OslCompletion::new().unwrap(),
            #[cfg(feature = "linux")]
            xfer: Arc::pin_init(new_spinlock!(MasterXfer::default())).unwrap(),
            #[cfg(feature = "arceos")]
            xfer: new_spinlock!(MasterXfer::default()),
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
        self.master_setup();
        self.driver.disable_all_interrupt();
        Ok(())
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

        // On AMD pltforms BIOS advertises the bus clear feature
        // and enables the SCL/SDA stuck low. SMU FW does the
        // bus recovery process. Driver should not ignore this BIOS
        // advertisement of bus clear feature.
        if self.driver.ic_con().is_set(IC_CON::BUS_CLEAR_FEATURE_CTRL) {
            self.cfg.modify(IC_CON::BUS_CLEAR_FEATURE_CTRL.val(1));
        }

        self.driver.cfg_init_speed(&mut self.cfg);
        Ok(())
    }

    /// return  i2c functionality
    pub fn get_functionality(&self) -> I2cFuncFlags {
        self.driver.functionality
    }

    /// Prepare controller for a transaction and call xfer_msg
    pub fn master_transfer(&self, msgs: Vec<I2cMsg>) -> Result<i32> {
        let msg_num = msgs.len();
        // reinit complete
        self.cmd_complete.reinit();
        // wait bus free
        self.driver.wait_bus_not_busy()?;
        // transfer exit make sure interrupt is disabled 
        // so here lock is safety
        let mut transfer = self.xfer.lock();
        transfer.prepare(msgs, &self);
        drop(transfer);
        // Now, could enable interrupt
        self.driver.clear_all_interrupt();
        self.driver.write_interrupt_mask(&Self::master_default_intr_mask());

        // wait transfer complete
        match self.cmd_complete.wait_for_completion_timeout(1) {
            Err(e) => {
                log_err!("wait complete timeout");
                //master_setup implicitly disables the adapter
                self.master_setup();
                self.driver.clear_all_interrupt();
                self.driver.disable_all_interrupt();
                return Err(e);
            }
            Ok(_) => (),
        }

        // complete make sure interrupt is disable 
        // so here lock is safety
        let mut transfer = self.xfer.lock();
        transfer.exit(&self)?;
        Ok(msg_num.try_into().unwrap())
    }
    
    /// Interrupt service routine. This gets called whenever an I2C master interrupt
    /// occurs
    pub fn irq_handler(&self) -> irq::Return {
        let enable = self.driver.ic_enable();
        let stat = self.driver.ic_raw_intr_stat().get();
        // check raw intr stat
        if !enable.is_set(IC_ENABLE::ENABLE) || (stat & !0b100000000) == 0 {
            return to_irq_return(ReturnEnum::None);
        }

        // master_transfer make sure when irq hanppend(irq enable)
        // no longer lock transfer, so here lock is safety
        log_debug!("enter irq stat: {:x}, enable: {:x}", stat, enable.get());
        let mut transfer = self.xfer.lock();
        let result = transfer.irq_process(&self);
        drop(transfer);

        match result {
            TransferResult::UnExpectedInterrupt => {
                self.driver.disable_all_interrupt();
            },
            TransferResult::Abort => {
                // Anytime TX_ABRT is set, the contents of the tx/rx
                // buffers are flushed. Make sure to skip them.
                self.driver.disable_all_interrupt();
                self.cmd_complete.complete();
            },
            TransferResult::Fininsh => {
                self.cmd_complete.complete();
            },
            TransferResult::Continue => (),
        }
        return to_irq_return(ReturnEnum::Handled);
    }

    fn master_setup(&self) {
        // Disable the adapter
        self.disable(false);
        // Write standard speed timing parameters
        self.driver.write_lhcnt(&self.lhcnt);
        // Write SDA hold time if supported
        self.driver.write_sda_hold_time();
        // Write fifo
        self.driver.write_fifo(self.tx_fifo_depth / 2, 0);
        // set IC_CON
        self.driver.write_ic_con(&self.cfg);
    }

    fn disable(&self, fast: bool) {
        if fast {
            self.driver.disable_nowait();
        } else {
            self.driver.disable_controler();
        }
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
                self.lhcnt.fs_hcnt,
                self.lhcnt.fs_lcnt
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

    fn master_default_intr_mask() -> LocalRegisterCopy<u32, IC_INTR::Register> {
        let mut mask = LocalRegisterCopy::new(0);
        mask.modify(IC_INTR::RX_FULL.val(0b1));
        mask.modify(IC_INTR::TX_ABRT.val(0b1));
        mask.modify(IC_INTR::STOP_DET.val(0b1));
        mask.modify(IC_INTR::TX_EMPTY.val(0b1));
        mask
    }
}
