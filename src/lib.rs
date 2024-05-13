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
    interfaces::{Readable, Writeable,ReadWriteable},
    LocalRegisterCopy,
};

pub(crate) mod common;
pub(crate) mod core;
mod master;
pub(crate) mod registers;

use crate::{
    common::{DwI2cStatus,DwI2cSclLHCnt}, 
    core::*, 
    registers::*
};

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
            speed_mode: I2cSpeedMode::StandMode,
            mode: I2cMode::Master,
            status: DwI2cStatus::empty(),
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn mode_init(&mut self, mode: I2cMode) {
        self.mode = mode;
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

    #[inline]
    pub(crate) fn functionality_init(&mut self, functionality: I2cFuncFlags) {
        self.functionality |= functionality;
    }

    #[inline]
    pub(crate) fn ic_rxflr(&mut self) -> LocalRegisterCopy<u32, IC_GENERAL_FLR::Register> {
        self.regs.IC_RXFLR.extract()
    }

    #[inline]
    pub(crate) fn ic_comp_param_1(&mut self) -> LocalRegisterCopy<u32, IC_COMP_PARAM_1::Register> {
        self.regs.IC_COMP_PARAM_1.extract()
    }

    #[inline]
    pub(crate) fn ic_con(&mut self) -> LocalRegisterCopy<u32, IC_CON::Register> {
        self.regs.IC_CON.extract()
    }

    pub(crate) fn cfg_init_speed(&mut self, cfg: &mut LocalRegisterCopy<u32, IC_CON::Register>) {
        match self.speed_mode {
            I2cSpeedMode::StandMode => cfg.modify(IC_CON::SPEED.val(0b01)),
            I2cSpeedMode::HighSpeedMode => cfg.modify(IC_CON::SPEED.val(0b11)),
            _ => cfg.modify(IC_CON::SPEED.val(0b10)),
        }
    }

    #[inline]
    pub(crate) fn set_ic_con(&mut self, cfg: &LocalRegisterCopy<u32, IC_CON::Register>) {
        self.regs.IC_CON.set(cfg.get());
    }

    #[inline]
    pub(crate) fn enable_10bitaddr(&mut self, enable: bool) {
        if enable {
            self.regs.IC_CON.modify(IC_CON::IC_10BITADDR_MASTER.val(0b1));
        } else {
            self.regs.IC_CON.modify(IC_CON::IC_10BITADDR_MASTER.val(0b0));
        }
    }

    #[inline]
    pub(crate) fn set_ic_tar(&mut self, ic_tar: u32) {
        self.regs.IC_TAR.set(tar);
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

    pub(crate) fn set_lhcnt(&mut self, lhcnt: &DwI2cSclLHCnt) {
        // Write standard speed timing parameters
        self.regs.IC_SS_OR_UFM_SCL_LCNT.set(lhcnt.ss_lcnt.into());
        self.regs.IC_SS_OR_UFM_SCL_HCNT.set(lhcnt.ss_hcnt.into());

        // Write fast mode/fast mode plus timing parameters
        self.regs.IC_FS_SCL_LCNT.set(lhcnt.fs_lcnt.into());
        self.regs.IC_FS_SCL_HCNT_OR_UFM_TBUF_CNT.set(lhcnt.fs_hcnt.into());

        // Write high speed timing parameters if supported
        if self.speed_mode == I2cSpeedMode::HighSpeedMode {
            self.regs.IC_HS_SCL_LCNT.set(lhcnt.hs_lcnt.into());
            self.regs.IC_HS_SCL_HCNT.set(lhcnt.hs_hcnt.into());
        }
    }

    #[inline]
    pub(crate) fn set_fifo(&mut self, ic_tx: u32, ic_rx:u32) {
        self.regs.IC_TX_TL.set(ic_tx);
        self.regs.IC_RX_TL.set(ic_rx);
    }

    pub(crate) fn wait_bus_not_busy(&mut self) -> Result<()> {
        if let Err(e) = read_poll_timeout(
            || return self.regs.IC_STATUS.extract(),
            move |x| !x.is_set(IC_STATUS::ACTIVITY),
            1100,
            20000,
            false,
        ) {
            log_err!("{:?} while waiting for bus ready", e);
            return to_error(Errno::Busy);
        }
        Ok(())

        //TODO: bus recovery
    }

    #[inline]
    pub(crate) fn ic_enable(&mut self) -> LocalRegisterCopy<u32, IC_ENABLE::Register> {
        self.regs.IC_ENABLE.extract()
    }

    #[inline]
    pub(crate) fn ic_enable_status(&mut self) -> LocalRegisterCopy<u32, IC_ENABLE_STATUS::Register> {
        self.regs.IC_ENABLE_STATUS.extract()
    }

    #[inline]
    pub(crate) fn ic_raw_intr_stat(&mut self) -> LocalRegisterCopy<u32, IC_INTR::Register> {
        self.regs.IC_RAW_INTR_STAT.extract()
    }

    pub(crate) fn read_and_clean_intrbits(&mut self, abort_source: &mut u32, rx_outstanding: u32) 
        -> LocalRegisterCopy<u32, IC_INTR::Register> {
        // The IC_INTR_STAT register just indicates "enabled" interrupts. 
        // The unmasked raw version of interrupt status bits is available
        // in the IC_RAW_INTR_STAT register.
        //
        // That is,
        // stat = readl(IC_INTR_STAT);
        // equals to,
        // stat = readl(IC_RAW_INTR_STAT) & readl(IC_INTR_MASK);
        // The raw version might be useful for debugging purposes.
        let stat = self.regs.IC_INTR_STAT.extract();
        
        // Do not use the IC_CLR_INTR register to clear interrupts, or
        // you'll miss some interrupts, triggered during the period from
        // readl(IC_INTR_STAT) to readl(IC_CLR_INTR).
        // Instead, use the separately-prepared IC_CLR_* registers.

        if stat.is_set(IC_INTR::RX_UNDER) {
            let _ = self.regs.IC_CLR_RX_UNDER.get();
        }
        if stat.is_set(IC_INTR::RX_OVER) {
            let _ = self.regs.IC_CLR_RX_OVER.get();
        }
        if stat.is_set(IC_INTR::TX_OVER) {
            let _ = self.regs.IC_CLR_TX_OVER.get();
        }
        if stat.is_set(IC_INTR::RD_REQ) {
            let _ = self.regs.IC_CLR_RD_REQ.get();
        }
        if stat.is_set(IC_INTR::TX_ABRT) {
            // The IC_TX_ABRT_SOURCE register is cleared whenever
            // the IC_CLR_TX_ABRT is read.  Preserve it beforehand.
            *abort_source = self.regs.IC_TX_ABRT_SOURCE.get();
            let _ = self.regs.IC_CLR_TX_ABRT.get();
        }
        if stat.is_set(IC_INTR::RX_DONE) {
            let _ = self.regs.IC_CLR_RX_DONE.get();
        }
        if stat.is_set(IC_INTR::ACTIVITY) {
            let _ = self.regs.IC_CLR_ACTIVITY.get();
        }
        if stat.is_set(IC_INTR::STOP_DET) {
            if rx_outstanding == 0 || stat.is_set(IC_INTR::RX_FULL)  {
                let _ = self.regs.IC_CLR_STOP_DET.get();
            }
        }
        if stat.is_set(IC_INTR::START_DET) {
            let _ = self.regs.IC_CLR_START_DET.get();
        }
        if stat.is_set(IC_INTR::GEN_CALL) {
            let _ = self.regs.IC_CLR_GEN_CALL.get();
        }
        stat
    }

    #[inline]
    pub(crate) fn set_interrupt_mask(&mut self, mask: &LocalRegisterCopy<u32, IC_INTR::Register>) {
        self.regs.IC_INTR_MASK.set(mask.get());
    }

    #[inline]
    pub(crate) fn disable_all_interrupt(&mut self) {
        self.regs.IC_INTR_MASK.set(0);
    }

    #[inline]
    pub(crate) fn clear_all_interrupt(&mut self) {
        self.regs.IC_CLR_INTR.get();
    }

    pub(crate) fn disable(&mut self) {
        self.disable_controler();
        // Disable all interrupts
        self.disable_all_interrupt();
        self.clear_all_interrupt();
    }

    pub(crate) fn enable_controler(&mut self) {
        self.regs.IC_ENABLE.set(1);
        self.status |= DwI2cStatus::ACTIVE;
    }

    pub(crate) fn is_active(&self) -> bool {
        self.status.contains(DwI2cStatus::ACTIVE)
    }

    pub(crate) fn disable_controler(&mut self) {
        let raw_int_stat = self.regs.IC_RAW_INTR_STAT.extract();
        let mut ic_enable = self.ic_enable();

        let need_aborted = raw_int_stat.is_set(IC_INTR::MST_ON_HOLD);
        if need_aborted {
            ic_enable.modify(IC_ENABLE::ABORT.val(1));
            self.regs.IC_ENABLE.set(ic_enable.get());

            if let Err(e) = read_poll_timeout(
                || return self.ic_enalbe(),
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
            if !self.ic_enable_status().is_set(IC_ENABLE_STATUS::IC_EN) {
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
