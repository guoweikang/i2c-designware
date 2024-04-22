///! I2C Time configuration

use core::mem::MaybeUninit;
use crate::I2cSpeedMode;

/// I2C standard mode max bus frequency in hz
pub const I2C_MAX_STANDARD_MODE_FREQ:u32 = 100000;
/// I2C fast mode max bus frequency in hz
pub const I2C_MAX_FAST_MODE_FREQ:u32 = 400000;
/// I2C fast plus mode max bus frequency in hz
pub const I2C_MAX_FAST_MODE_PLUS_FREQ:u32 = 1000000;
/// I2C turbo mode max bus frequency in hz
pub const I2C_MAX_TURBO_MODE_FREQ:u32 = 1400000;
/// I2C high speed mode max bus frequency in hz
pub const I2C_MAX_HIGH_SPEED_MODE_FREQ:u32 = 3400000;
/// I2C ultra fast mode max bus frequency in hz
pub const I2C_MAX_ULTRA_FAST_MODE_FREQ:u32 = 5000000;

/// I2C timing config for all i2c driver
/// 
/// An instance of `I2cTiming` include can be used for any i2c driver to describe
/// the bus frequency in Hz
/// time SCL signal takes to rise in ns; t(r) in the I2C specification
/// time SCL signal takes to fall in ns; t(f) in the I2C specification
/// time IP core additionally needs to setup SCL in ns
/// time SDA signal takes to fall in ns; t(f) in the I2C specification
/// time IP core additionally needs to hold SDA in ns
/// width in ns of spikes on i2c lines that the IP core digital filter can filter out
/// threshold frequency for the low pass IP core analog filter
pub struct I2cTiming {
    bus_freq_hz: u32,
    scl_rise_ns: u32,
    scl_fall_ns: u32,
    scl_int_delay_ns: u32,
    sda_fall_ns: u32,
    sda_hold_ns: u32,
    digital_filter_width_ns: u32,
    analog_filter_cutoff_freq_hz: u32,
}

impl I2cTiming {

    /// Create a default timing configuration for a special SpeedMode 
    pub fn new(mode: I2cSpeedMode) -> I2cTiming {
        // SAFETY: The variables will be fully initialized later.
        let t: I2cTiming = unsafe { MaybeUninit::zeroed().assume_init() };    
        
        match mode {
            I2cSpeedMode::StandMode => t.bus_freq_hz(I2C_MAX_STANDARD_MODE_FREQ).scl_rise_ns(1000).scl_fall_ns(300),
            I2cSpeedMode::FastMode => t.bus_freq_hz(I2C_MAX_FAST_MODE_FREQ).scl_rise_ns(300).scl_fall_ns(300),
            I2cSpeedMode::FastPlusMode => t.bus_freq_hz(I2C_MAX_FAST_MODE_PLUS_FREQ).scl_rise_ns(120).scl_fall_ns(120),
            I2cSpeedMode::TurboMode => t.bus_freq_hz(I2C_MAX_TURBO_MODE_FREQ).scl_rise_ns(120).scl_fall_ns(120),
            I2cSpeedMode::HighSpeedMode => t.bus_freq_hz(I2C_MAX_HIGH_SPEED_MODE_FREQ).scl_rise_ns(120).scl_fall_ns(120),
            I2cSpeedMode::UltraFastMode => t.bus_freq_hz(I2C_MAX_ULTRA_FAST_MODE_FREQ).scl_rise_ns(120).scl_fall_ns(120),
        }
    }

    /// get bus freq HZ
    #[inline]
    pub fn get_bus_freq_hz(&self) -> u32 {
        self.bus_freq_hz
    }

    /// set bus_freq_hz and return self
    #[inline]
    pub fn bus_freq_hz(mut self, val: u32) -> Self {
        self.bus_freq_hz = val;
        self
    }

    /// set scl_rise_ns and return self
    #[inline]
    pub fn scl_rise_ns(mut self, val: u32) -> Self {
        self.scl_rise_ns = val;
        self
    }

    /// set scl_fall_ns and return self
    #[inline]
    pub fn scl_fall_ns(mut self, val: u32) -> Self {
        self.scl_fall_ns = val;
        self
    }

    /// set scl_int_delay and return self
    #[inline]
    pub fn scl_int_delay_ns(mut self, val: u32) -> Self {
        self.scl_int_delay_ns = val;
        self
    }

    /// set sda_fall_ns and return self
    #[inline]
    pub fn sda_fall_ns(mut self, val: u32) -> Self {
        self.sda_fall_ns = val;
        self
    }

    /// set sda_hold_ns and return self
    #[inline]
    pub fn sda_hold_ns(mut self, val: u32) -> Self {
        self.sda_hold_ns = val;
        self
    }

    /// set digital_filter_width_ns and return self
    #[inline]
    pub fn digital_filter_width_ns(mut self, val: u32) -> Self {
        self.digital_filter_width_ns = val;
        self
    }

    /// set analog_filter_cutoff_freq_hz and return self
    #[inline]
    pub fn analog_filter_cutoff_freq_hz(mut self, val: u32) -> Self {
        self.analog_filter_cutoff_freq_hz = val;
        self
    }
}





