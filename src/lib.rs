//! Driver for the  Synopsys DesignWare I2C
//!


#![no_std]

#![feature(const_option)]
#![feature(const_nonnull_new)]

pub(crate) mod registers;
pub(crate) mod common;

use core::ptr::NonNull;
use common::{timing, I2cMode};
use registers::DwApbI2cRegisters;

pub(crate) use osl::error::{Result, Error, Errno};  


/// I2cDesignwareDriverConfig
#[allow(dead_code)] // remove me
pub struct I2cDesignwareDriverConfig {
    mode: I2cMode,
    irq: u32,
    timing: timing::I2cTiming,
}

#[allow(dead_code)] // remove me
/// The I2cDesignware Driver
pub struct I2cDesignwareDriver {
    regs: NonNull<DwApbI2cRegisters>,
    config: I2cDesignwareDriverConfig,
}

const I2C_DESIGNWARE_SUPPORT_SPEED:[u32;4] = [
    timing::I2C_MAX_STANDARD_MODE_FREQ, 
    timing::I2C_MAX_FAST_MODE_FREQ, 
    timing::I2C_MAX_FAST_MODE_PLUS_FREQ,
    timing::I2C_MAX_HIGH_SPEED_MODE_FREQ,
]; 

impl I2cDesignwareDriver {

    /// Create a new I2cDesignwareDriver
    pub const fn new(config: I2cDesignwareDriverConfig, base_addr: *mut u8) -> I2cDesignwareDriver  {
        I2cDesignwareDriver {
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
        if !I2C_DESIGNWARE_SUPPORT_SPEED.contains(&self.config.timing.get_bus_freq_hz()) {
            return Err(Error::from(Errno::InvalidArgs));
        }
        
        Ok(())
    }
}
