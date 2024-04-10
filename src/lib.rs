//! Driver for the  Synopsys DesignWare I2C
//!

#![no_std]

pub(crate) mod registers;
pub(crate) mod osl;
pub mod common;

use common::*;
pub(crate) use osl::error::{Result, Error, Errno};
use registers::DwApbI2cRegisters;

pub struct I2cDesignwareDriverConfig {
    mode: I2cMode,
    irq: u32,
    timing: common::timing::I2cTiming,
}

/// The I2cDesignware Driver
pub struct I2cDesignwareDriver {
    regs: NonNull<DwApbI2cRegisters>,
    config: I2cDesignwareDriverConfig,
}

const I2C_DESIGNWARE_SUPPORT_SPEED:[u32;3] = [
    timing::I2C_MAX_STANDARD_MODE_FREQ, 
    timing::I2C_MAX_FAST_MODE_FREQ, 
    timing::I2C_MAX_FAST_MODE_PLUS_FREQ,
    timing::I2C_MAX_HIGH_SPEED_MODE_FREQ,
]; 

impl I2cDesignwareDriver {

    pub const fn new(config: I2cDesignwareDriverConfig, base_addr: usize) -> I2cDesignwareDriver  {
        I2cDesignwareDriver {
            config,
            regs: NonNull::new(base_addr),
        }
    }

    pub fn setup(&mut self) -> Result<()> {
        self.speed_check()?;

    }

    fn speed_check(&self) -> Result<()> {
        if !I2C_DESIGNWARE_SUPPORT_SPEED.contains(self.config.get_bus_freq_hz()) {
            Err(Errno::InvalidArgs)
        }
        
        Ok(())
    }
}
