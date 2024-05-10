use i2c_common::*;
use osl::error::Result;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{common::DwI2cSclLHCnt, registers::*, I2cDwCoreDriver, I2cDwDriverConfig};

/// The I2cDesignware Driver
#[allow(dead_code)]
pub struct I2cDwMasterDriver {
    /// core Driver
    driver: I2cDwCoreDriver,
    /// I2c scl_LHCNT
    lhcnt: DwI2cSclLHCnt,
    /// Fifo
    tx_fifo_depth: u32,
    rx_fifo_depth: u32,
}

unsafe impl Sync for I2cDwMasterDriver {}
unsafe impl Send for I2cDwMasterDriver {}

impl I2cDwMasterDriver {
    /// Create a new I2cDesignwarDriver
    pub fn new(config: I2cDwDriverConfig, base_addr: *mut u8) -> I2cDwMasterDriver {
        Self {
            driver: I2cDwCoreDriver::new(config, base_addr),
            lhcnt: DwI2cSclLHCnt::default(),
            tx_fifo_depth: 0,
            rx_fifo_depth: 0,
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
    /*
     pub fn xfer(&mut self) {


     }

     fn xfer_init(&mut self) {
         self.driver.disable_controler();
     }
    */
    /// functionality and cfg init
    fn config_init(&mut self) -> Result<()> {
        // init functionality
        let functionality = I2cFuncFlags::TEN_BIT_ADDR;
        self.driver.functionality_init(functionality);

        // init master cfg
        self.driver.cfg.modify(IC_CON::MASTER_MODE.val(1));
        self.driver.cfg.modify(IC_CON::IC_SLAVE_DISABLE.val(1));
        self.driver.cfg.modify(IC_CON::IC_RESTART_EN.val(1));

        // On AMD platforms BIOS advertises the bus clear feature
        // and enables the SCL/SDA stuck low. SMU FW does the
        // bus recovery process. Driver should not ignore this BIOS
        // advertisement of bus clear feature.
        if self
            .driver
            .regs
            .IC_CON
            .is_set(IC_CON::BUS_CLEAR_FEATURE_CTRL)
        {
            self.driver
                .cfg
                .modify(IC_CON::BUS_CLEAR_FEATURE_CTRL.val(1));
        }

        self.driver.cfg_init();
        Ok(())
    }

    fn master_setup(&mut self) -> Result<()> {
        // Disable the adapter
        self.driver.disable_controler();
        // Write standard speed timing parameters
        self.driver
            .regs
            .IC_SS_OR_UFM_SCL_LCNT
            .set(self.lhcnt.ss_lcnt.into());
        self.driver
            .regs
            .IC_SS_OR_UFM_SCL_HCNT
            .set(self.lhcnt.ss_hcnt.into());

        // Write fast mode/fast mode plus timing parameters
        self.driver
            .regs
            .IC_FS_SCL_LCNT
            .set(self.lhcnt.fs_lcnt.into());
        self.driver
            .regs
            .IC_FS_SCL_HCNT_OR_UFM_TBUF_CNT
            .set(self.lhcnt.fs_hcnt.into());

        // Write high speed timing parameters if supported
        if self.driver.speed_mode == I2cSpeedMode::HighSpeedMode {
            self.driver
                .regs
                .IC_HS_SCL_LCNT
                .set(self.lhcnt.hs_lcnt.into());
            self.driver
                .regs
                .IC_HS_SCL_HCNT
                .set(self.lhcnt.hs_hcnt.into());
        }

        // Write SDA hold time if supported
        self.driver.write_sda_hold_time();
        // Write fifo
        self.driver.regs.IC_TX_TL.set(self.tx_fifo_depth / 2);
        self.driver.regs.IC_RX_TL.set(0);

        // set IC_CON
        self.driver.write_cfg();
        Ok(())
    }

    fn fifo_size_init(&mut self) {
        let com_param_1 = self.driver.regs.IC_COMP_PARAM_1.extract();
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
