use osl::error::{to_error, Errno, Result};
use tock_registers::interfaces::Readable;
use i2c_common::*;

use crate::registers::{DwApbI2cRegistersRef,DW_IC_COMP_TYPE_VALUE};
use crate::I2cDwDriverConfig;
use crate::core::*;

/// The I2cDesignware Driver
#[allow(dead_code)]
pub struct I2cDwMasterDriver {
    regs: DwApbI2cRegistersRef,
    /// Config From external
    ext_config: I2cDwDriverConfig,
    functionality:Option<I2cFuncFlags>,
    cfg: Option<DwI2cConfigFlags>,
}

const I2C_DESIGNWARE_SUPPORT_SPEED: [u32; 4] = [
    I2C_MAX_STANDARD_MODE_FREQ,
    I2C_MAX_FAST_MODE_FREQ,
    I2C_MAX_FAST_MODE_PLUS_FREQ,
    I2C_MAX_HIGH_SPEED_MODE_FREQ,
];

impl I2cDwMasterDriver {
    /// Create a new I2cDesignwarDriver
    pub const fn new(config: I2cDwDriverConfig, base_addr: *mut u8) -> I2cDwMasterDriver {
        I2cDwMasterDriver {
            ext_config: config,
            regs: DwApbI2cRegistersRef::new(base_addr),
            functionality: None,
            cfg: None,
        }
    }

    /// init I2cDwMasterDriver config ,call only once
    pub fn config_init(&mut self) -> Result<()> {
        self.speed_check()?;

        // init functionality
        self.functionality = Some(I2cFuncFlags::TEN_BIT_ADDR | DW_I2C_DEFAULT_FUNCTIONALITY);
        // init master cfg flags
        let mut master_cfg = DwI2cConfigFlags::MASTER | DwI2cConfigFlags::SLAVE_DISABLE | 
            DwI2cConfigFlags::RESTART_EN;
        
        master_cfg |= match self.ext_config.timing.get_bus_freq_hz() {
            I2C_MAX_STANDARD_MODE_FREQ => DwI2cConfigFlags::SPEED_STD,
            I2C_MAX_HIGH_SPEED_MODE_FREQ => DwI2cConfigFlags::SPEED_HIGH,
            _ => DwI2cConfigFlags::SPEED_FAST,
        };
        self.cfg = Some(master_cfg);
        Ok(())
    }

    /// Initialize the designware I2C master hardware
    pub fn setup(&mut self) -> Result<()> {
        self.com_type_check()?;
        //self.timing_setup()?;
        Ok(())
    }

    /*
    fn timing_setup(&mut self) -> Result<()> {
        let com_param = self.regs.IC_COMP_PARAM_1.get();
        let mut scl_fall_ns = self.ext_config.timing.get_scl_fall_ns();
        let mut sda_fall_ns = self.ext_config.timing.get_sda_fall_ns();
    
        // Set standard and fast speed dividers for high/low periods
        if scl_fall_ns == 0 {
            scl_fall_ns = 300;
        }

        if sda_fall_ns == 0 {
            sda_fall_ns = 300;
        }

        Ok(())

    }
*/
    fn com_type_check(&mut self) -> Result<()> {
        let com_type = self.regs.IC_COMP_TYPE.get();
        if com_type == DW_IC_COMP_TYPE_VALUE {
            log_info!("com_type check Ok");
        } else if com_type == DW_IC_COMP_TYPE_VALUE & 0x0000ffff { 
            log_error!("com_type check Failed, not support 16 bit system ");
            return to_error(Errno::NoSuchDevice);
        } else if com_type == DW_IC_COMP_TYPE_VALUE.to_be() {
            log_error!("com_type check Failed, not support BE system ");
            return to_error(Errno::NoSuchDevice);
        } else {
            log_error!("com_type check failed, Unknown Synopsys component type: {:x}", com_type);
            return to_error(Errno::NoSuchDevice);
        }
        Ok(())
    }

    fn speed_check(&self) -> Result<()> {
        let bus_freq_hz = self.ext_config.timing.get_bus_freq_hz();
        if !I2C_DESIGNWARE_SUPPORT_SPEED.contains(&bus_freq_hz) {
            log_error!("{bus_freq_hz} Hz is unsupported, only 100kHz, 400kHz, 1MHz and 3.4MHz are supported");
            return to_error(Errno::InvalidArgs);
        }
        Ok(())
    }
}
