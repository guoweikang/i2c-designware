use core::ptr::NonNull;
use osl::error::{to_error, Errno, Result};

use crate::timing;
use crate::registers::DwApbI2cRegisters;
use crate::I2cDwDriverConfig;

/// The I2cDesignware Driver
#[allow(dead_code)]
pub struct I2cDwMasterDriver {
    regs: NonNull<DwApbI2cRegisters>,
    config: I2cDwDriverConfig,
}

const I2C_DESIGNWARE_SUPPORT_SPEED: [u32; 4] = [
    timing::I2C_MAX_STANDARD_MODE_FREQ,
    timing::I2C_MAX_FAST_MODE_FREQ,
    timing::I2C_MAX_FAST_MODE_PLUS_FREQ,
    timing::I2C_MAX_HIGH_SPEED_MODE_FREQ,
];

impl I2cDwMasterDriver {
    /// Create a new I2cDesignwarDriver
    pub const fn new(config: I2cDwDriverConfig, base_addr: *mut u8) -> I2cDwMasterDriver {
        I2cDwMasterDriver {
            config,
            regs: NonNull::new(base_addr).expect("ptr is null").cast(),
        }
    }

    /// init I2cDesignwareDriver,call only once
    pub fn setup(&mut self) -> Result<()> {
        self.speed_check()?;
        Ok(())
    }

    fn speed_check(&self) -> Result<()> {
        let bus_freq_hz = self.config.timing.get_bus_freq_hz();
        if !I2C_DESIGNWARE_SUPPORT_SPEED.contains(&bus_freq_hz) {
            log_error!("{bus_freq_hz} Hz is unsupported, only 100kHz, 400kHz, 1MHz and 3.4MHz are supported");
            return to_error(Errno::InvalidArgs);
        }
        Ok(())
    }
}
