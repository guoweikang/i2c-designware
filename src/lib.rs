//! Driver for the  Synopsys DesignWare I2C
//!

#![no_std]
#![feature(const_option)]
#![feature(const_nonnull_new)]

pub(crate) mod common;
pub(crate) mod registers;

pub use crate::common::{timing::I2cTiming, I2cMode, I2cSpeedMode};

use crate::common::timing;
use core::ptr::NonNull;
use registers::DwApbI2cRegisters;

pub(crate) use osl::error::{to_error, Errno, Result};

/// I2cDesignwareDriverConfig
pub struct I2cDesignwareDriverConfig {
    mode: I2cMode,
    irq: i32,
    timing: I2cTiming,
}

impl I2cDesignwareDriverConfig {
    pub fn new(mode: I2cMode, irq: i32, timing: I2cTiming) -> Self {
        Self { mode, irq, timing }
    }
}

/// The I2cDesignware Driver
pub struct I2cDesignwareDriver {
    regs: NonNull<DwApbI2cRegisters>,
    config: I2cDesignwareDriverConfig,
}

const I2C_DESIGNWARE_SUPPORT_SPEED: [u32; 4] = [
    timing::I2C_MAX_STANDARD_MODE_FREQ,
    timing::I2C_MAX_FAST_MODE_FREQ,
    timing::I2C_MAX_FAST_MODE_PLUS_FREQ,
    timing::I2C_MAX_HIGH_SPEED_MODE_FREQ,
];

impl I2cDesignwareDriver {
    /// Create a new I2cDesignwareDriver
    pub const fn new(config: I2cDesignwareDriverConfig, base_addr: *mut u8) -> I2cDesignwareDriver {
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
            return to_error(Errno::InvalidArgs);
        }

        Ok(())
    }
}
