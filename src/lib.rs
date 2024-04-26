//! Driver for the  Synopsys DesignWare I2C
//!

#![no_std]
#![feature(const_option)]
#![feature(const_nonnull_new)]
#![feature(const_trait_impl)]

#[macro_use]
extern crate derive_builder;

#[macro_use]
extern crate osl;

pub mod common;
pub(crate) mod registers;
pub(crate) mod core;

pub use crate::common::{
    timing, timing::I2cTiming, timing::I2cTimingBuilder, I2cMode, I2cSpeedMode,
};
pub use crate::common::functionality::*;

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
