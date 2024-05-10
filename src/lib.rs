//! Driver for the  Synopsys DesignWare I2C
//!

#![no_std]
#![feature(const_option)]
#![feature(const_nonnull_new)]
#![feature(const_trait_impl)]

#[macro_use]
extern crate osl;

use i2c_common::*;
use osl::error::{to_error, Errno, Result};
use osl::sleep::usleep;
use tock_registers::{
    interfaces::{Readable, Writeable},
    LocalRegisterCopy,
};

pub(crate) mod common;
pub(crate) mod core;
mod master;
pub(crate) mod registers;

use crate::{common::DwI2cStatus, core::*, registers::*};

/// I2cDwDriverConfig
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct I2cDwDriverConfig {
    irq: i32,
    timing: I2cTiming,
    clk_rate_khz: u32,
}

impl I2cDwDriverConfig {
    /// Create  a Config
    pub fn new(irq: i32, timing: I2cTiming, clk_rate_khz: u32) -> Self {
        Self {
            irq,
            timing,
            clk_rate_khz,
        }
    }
}

pub use crate::master::I2cDwMasterDriver;

/// The I2cDesignware Core Driver
#[allow(dead_code)]
pub(crate) struct I2cDwCoreDriver {
    /// I2c Registers
    pub(crate) regs: DwApbI2cRegistersRef,
    /// Config From external
    pub(crate) ext_config: I2cDwDriverConfig,
    /// Corrected bus_freq_hz
    pub(crate) bus_freq_hz: u32,
    /// Corrected sda_hold_time
    pub(crate) sda_hold_time: Option<u32>,

    /// I2c functionality
    pub(crate) functionality: I2cFuncFlags,
    /// I2c Config  register set value
    pub(crate) cfg: LocalRegisterCopy<u32, IC_CON::Register>,

    /// I2c SpeedMode
    speed_mode: I2cSpeedMode,

    /// I2c Master or Slave mode
    pub(crate) mode: I2cMode,

    /// Driver Status
    status: DwI2cStatus,
}

unsafe impl Sync for I2cDwCoreDriver {}
unsafe impl Send for I2cDwCoreDriver {}

const I2C_DESIGNWARE_SUPPORT_SPEED: [u32; 4] = [
    I2C_MAX_STANDARD_MODE_FREQ,
    I2C_MAX_FAST_MODE_FREQ,
    I2C_MAX_FAST_MODE_PLUS_FREQ,
    I2C_MAX_HIGH_SPEED_MODE_FREQ,
];

impl I2cDwCoreDriver {
    pub(crate) fn new(config: I2cDwDriverConfig, base_addr: *mut u8) -> I2cDwCoreDriver {
        I2cDwCoreDriver {
            ext_config: config,
            regs: DwApbI2cRegistersRef::new(base_addr),
            bus_freq_hz: 0,
            sda_hold_time: None,
            functionality: DW_I2C_DEFAULT_FUNCTIONALITY,
            cfg: LocalRegisterCopy::new(0),
            speed_mode: I2cSpeedMode::StandMode,
            mode: I2cMode::Master,
            status: DwI2cStatus::empty(),
        }
    }

    pub(crate) fn speed_check(&mut self) -> Result<()> {
        let bus_freq_hz = self.ext_config.timing.get_bus_freq_hz();

        if !I2C_DESIGNWARE_SUPPORT_SPEED.contains(&bus_freq_hz) {
            log_err!("{bus_freq_hz} Hz is unsupported, only 100kHz, 400kHz, 1MHz and 3.4MHz are supported");
            return to_error(Errno::InvalidArgs);
        }
        self.bus_freq_hz = bus_freq_hz;

        if !self
            .regs
            .IC_COMP_PARAM_1
            .is_set(IC_COMP_PARAM_1::MAX_SPEED_MODE)
            && self.bus_freq_hz == I2C_MAX_HIGH_SPEED_MODE_FREQ
        {
            log_err!("High Speed not supported! Fall back to fast mode");
            self.bus_freq_hz = I2C_MAX_FAST_MODE_FREQ;
        }

        self.speed_mode = I2cSpeedMode::from_bus_freq(self.bus_freq_hz);
        Ok(())
    }

    pub(crate) fn com_type_check(&mut self) -> Result<()> {
        let com_type = self.regs.IC_COMP_TYPE.get();
        if com_type == DW_IC_COMP_TYPE_VALUE {
            log_info!("com_type check Ok");
        } else if com_type == DW_IC_COMP_TYPE_VALUE & 0x0000ffff {
            log_err!("com_type check Failed, not support 16 bit system ");
            return to_error(Errno::NoSuchDevice);
        } else if com_type == DW_IC_COMP_TYPE_VALUE.to_be() {
            log_err!("com_type check Failed, not support BE system ");
            return to_error(Errno::NoSuchDevice);
        } else {
            log_err!(
                "com_type check failed, Unknown Synopsys component type: {:x}",
                com_type
            );
            return to_error(Errno::NoSuchDevice);
        }
        Ok(())
    }

    pub(crate) fn functionality_init(&mut self, functionality: I2cFuncFlags) {
        self.functionality |= functionality;
    }

    pub(crate) fn cfg_init(&mut self) {
        match self.speed_mode {
            I2cSpeedMode::StandMode => self.cfg.modify(IC_CON::SPEED.val(0b01)),
            I2cSpeedMode::HighSpeedMode => self.cfg.modify(IC_CON::SPEED.val(0b11)),
            _ => self.cfg.modify(IC_CON::SPEED.val(0b10)),
        }
    }

    pub(crate) fn write_cfg(&mut self) {
        self.regs.IC_CON.set(self.cfg.get());
    }

    #[allow(dead_code)]
    pub(crate) fn mode_init(&mut self, mode: I2cMode) {
        self.mode = mode;
    }

    pub(crate) fn write_sda_hold_time(&mut self) {
        if self.sda_hold_time.is_some() {
            self.regs
                .IC_SDA_HOLD
                .set(*self.sda_hold_time.as_mut().unwrap());
        }
    }

    pub(crate) fn sda_hold_time_init(&mut self) -> Result<()> {
        let comp_ver = self.regs.IC_COMP_VERSION.get();
        let ext_sda_hold_time = self.ext_config.timing.get_sda_hold_ns();

        if comp_ver < DW_IC_SDA_HOLD_MIN_VERS {
            log_warn!("Hardware too old to adjust SDA hold time.");
            self.sda_hold_time = None;
            return Ok(());
        }

        let mut sda_hold_time = self.regs.IC_SDA_HOLD.extract();
        if ext_sda_hold_time == 0 {
            // Workaround for avoiding TX arbitration lost in case I2C
            // slave pulls SDA down "too quickly" after falling edge of
            // SCL by enabling non-zero SDA RX hold. Specification says it
            // extends incoming SDA low to high transition while SCL is
            // high but it appears to help also above issue.
            if !sda_hold_time.is_set(IC_SDA_HOLD::SDA_RX_HOLD) {
                sda_hold_time.write(IC_SDA_HOLD::SDA_TX_HOLD.val(1));
            }
            self.sda_hold_time = Some(sda_hold_time.get());
        } else {
            self.sda_hold_time = Some(ext_sda_hold_time);
        }

        sda_hold_time.set(*self.sda_hold_time.as_ref().unwrap());

        log_info!(
            "sda hold time Tx:Rx =  {}:{}",
            sda_hold_time.read(IC_SDA_HOLD::SDA_TX_HOLD),
            sda_hold_time.read(IC_SDA_HOLD::SDA_RX_HOLD)
        );

        log_info!("I2C  Bus Speed: {}", self.speed_mode);
        Ok(())
    }

    pub(crate) fn disable(&mut self) {
        self.disable_controler();
        // Disable all interrupts
        self.regs.IC_INTR_MASK.set(0);
        self.regs.IC_CLR_INTR.get();
    }

    pub(crate) fn wait_bus_not_busy(&mut self) {
        if let Err(e) = Self::read_poll_timeout(
            || return self.regs.IC_STATUS.extract(),
            move |x| !x.is_set(IC_STATUS::ACTIVITY),
            1100,
            20000,
            false,
        ) {
            log_err!("{:?} while waiting for bus ready", e);
        }

        //TODO: bus recovery
    }

    /// Poll until a condition is met or a timeout occurs
    fn read_poll_timeout<T, F: Fn() -> T, C: Fn(T) -> bool>(
        read_op: F,
        cond: C,
        sleep_us: u64,
        timeout_us: u64,
        sleep_before: bool,
    ) -> Result<()> {
        let timeout: u64 = osl::time::time_add_us(timeout_us);

        if sleep_us != 0 && sleep_before {
            osl::sleep::usleep(sleep_us);
        }

        let ret = loop {
            if cond(read_op()) {
                return Ok(());
            }

            if timeout_us != 0 && osl::time::current_time() > timeout {
                break read_op();
            }

            if sleep_us > 0 {
                osl::sleep::usleep(sleep_us);
            }
        };

        if cond(ret) {
            return Ok(());
        } else {
            return to_error(Errno::TimeOut);
        }
    }

    pub(crate) fn disable_controler(&mut self) {
        let raw_int_stat = self.regs.IC_RAW_INTR_STAT.extract();
        let mut ic_enable = self.regs.IC_ENABLE.extract();

        let need_aborted = raw_int_stat.is_set(IC_INTR::MST_ON_HOLD);
        if need_aborted {
            ic_enable.modify(IC_ENABLE::ABORT.val(1));
            self.regs.IC_ENABLE.set(ic_enable.get());

            if let Err(e) = Self::read_poll_timeout(
                || return self.regs.IC_ENABLE.extract(),
                move |x| !x.is_set(IC_ENABLE::ABORT),
                10,
                100,
                false,
            ) {
                log_err!("{:?} while trying to abort current transfer", e);
            }
        }

        let mut try_cnt = 100;
        loop {
            self.disable_nowait();
            usleep(100);
            // check enable_status
            if !self.regs.IC_ENABLE_STATUS.is_set(IC_ENABLE_STATUS::IC_EN) {
                log_info!("disable success");
                break;
            }
            try_cnt -= 1;
            if try_cnt == 0 {
                log_err!("timeout in disabling i2c adapter");
                break;
            }
        }
    }

    fn disable_nowait(&mut self) {
        self.regs.IC_ENABLE.set(0);
        self.status &= !DwI2cStatus::ACTIVE;
    }
}
