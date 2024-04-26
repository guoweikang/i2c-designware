//! Driver for the  Synopsys DesignWare I2C
//!

#![no_std]
#![feature(const_option)]
#![feature(const_nonnull_new)]

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate osl;

pub(crate) mod common;
pub(crate) mod registers;

pub use crate::common::{
    timing, timing::I2cTiming, timing::I2cTimingBuilder, I2cMode, I2cSpeedMode,
};

mod master;
pub use crate::master::I2cDwMasterDriver;

/// I2cDwDriverConfig
#[allow(dead_code)]
pub struct I2cDwDriverConfig {
    irq: i32,
    timing: I2cTiming,
}

impl I2cDwDriverConfig {
    /// Create  a Config
    pub fn new(irq: i32, timing: I2cTiming) -> Self {
        Self { irq, timing }
    }
}
