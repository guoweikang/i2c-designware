//! I2C Time configuration

use super::I2cSpeedMode;

/// I2C standard mode max bus frequency in hz
pub const I2C_MAX_STANDARD_MODE_FREQ: u32 = 100000;
/// I2C fast mode max bus frequency in hz
pub const I2C_MAX_FAST_MODE_FREQ: u32 = 400000;
/// I2C fast plus mode max bus frequency in hz
pub const I2C_MAX_FAST_MODE_PLUS_FREQ: u32 = 1000000;
/// I2C turbo mode max bus frequency in hz
pub const I2C_MAX_TURBO_MODE_FREQ: u32 = 1400000;
/// I2C high speed mode max bus frequency in hz
pub const I2C_MAX_HIGH_SPEED_MODE_FREQ: u32 = 3400000;
/// I2C ultra fast mode max bus frequency in hz
pub const I2C_MAX_ULTRA_FAST_MODE_FREQ: u32 = 5000000;

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
#[allow(missing_docs)]
#[derive(Clone, PartialEq, Eq, Debug, Default, Builder)]
#[builder(no_std)]
#[builder(build_fn(error(validation_error = false)))]
#[builder(public)]
pub struct I2cTiming {
    #[builder(default = "0")]
    bus_freq_hz: u32,
    #[builder(default = "0")]
    scl_rise_ns: u32,
    #[builder(default = "0")]
    scl_fall_ns: u32,
    #[builder(default = "0")]
    scl_int_delay_ns: u32,
    #[builder(default = "0")]
    sda_fall_ns: u32,
    #[builder(default = "0")]
    sda_hold_ns: u32,
    #[builder(default = "0")]
    digital_filter_width_ns: u32,
    #[builder(default = "0")]
    analog_filter_cutoff_freq_hz: u32,
}

impl I2cTiming {
    /// Create a default timing configuration for a special SpeedMode
    pub fn new_builder(mode: I2cSpeedMode, use_default: bool) -> I2cTimingBuilder {
        // SAFETY: The variables will be fully initialized later.
        let mut builder = I2cTimingBuilder::default();

        if use_default {
            match mode {
                I2cSpeedMode::StandMode => builder
                    .bus_freq_hz(I2C_MAX_STANDARD_MODE_FREQ)
                    .scl_rise_ns(1000)
                    .scl_fall_ns(300),
                I2cSpeedMode::FastMode => builder
                    .bus_freq_hz(I2C_MAX_FAST_MODE_FREQ)
                    .scl_rise_ns(300)
                    .scl_fall_ns(300),
                I2cSpeedMode::FastPlusMode => builder
                    .bus_freq_hz(I2C_MAX_FAST_MODE_PLUS_FREQ)
                    .scl_rise_ns(120)
                    .scl_fall_ns(120),
                I2cSpeedMode::TurboMode => builder
                    .bus_freq_hz(I2C_MAX_TURBO_MODE_FREQ)
                    .scl_rise_ns(120)
                    .scl_fall_ns(120),
                I2cSpeedMode::HighSpeedMode => builder
                    .bus_freq_hz(I2C_MAX_HIGH_SPEED_MODE_FREQ)
                    .scl_rise_ns(120)
                    .scl_fall_ns(120),
                I2cSpeedMode::UltraFastMode => builder
                    .bus_freq_hz(I2C_MAX_ULTRA_FAST_MODE_FREQ)
                    .scl_rise_ns(120)
                    .scl_fall_ns(120),
            };
        }
        builder
    }

    /// get bus freq HZ
    #[inline]
    pub fn get_bus_freq_hz(&self) -> u32 {
        self.bus_freq_hz
    }
}
