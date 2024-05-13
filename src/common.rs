use bitflags::bitflags;

#[allow(dead_code)]
#[derive(Default, Debug, Copy, Clone)]
pub(crate) struct DwI2cSclLHCnt {
    /// standard speed HCNT value
    pub(crate) ss_hcnt: u16,
    /// standard speed LCNT value
    pub(crate) ss_lcnt: u16,
    /// Fast Speed HCNT value
    pub(crate) fs_hcnt: u16,
    /// Fast Speed LCNT value
    pub(crate) fs_lcnt: u16,
    /// Fast Speed Plus HCNT value
    pub(crate) fp_hcnt: u16,
    /// Fast Speed Plus LCNT value
    pub(crate) fp_lcnt: u16,
    /// High Speed HCNT value
    pub(crate) hs_hcnt: u16,
    /// High Speed LCNT value
    pub(crate) hs_lcnt: u16,
}

impl DwI2cSclLHCnt {
    const MICRO: u64 = 1000000;

    fn div_round_closest_ull(x: u64, divisor: u64) -> u32 {
        ((x + divisor / 2) / divisor) as u32
    }

    /// Conditional expression:
    ///  
    ///  IC_[FS]S_SCL_LCNT + 1 >= IC_CLK * (tLOW + tf)
    ///
    /// DW I2C core starts counting the SCL CNTs for the LOW period
    /// of the SCL clock (tLOW) as soon as it pulls the SCL line.
    /// In order to meet the tLOW timing spec, we need to take into
    /// account the fall time of SCL signal (tf).  Default tf value
    /// should be 0.3 us, for safety.
    pub(crate) fn scl_lcnt(ic_clk: u32, tlow: u32, tf: u32, offset: u32) -> u32 {
        log_debug!(
            "scl_lcnt: ic_clk: {} , tlow:{}  tf:{} , offset:{}",
            ic_clk,
            tlow,
            tf,
            offset
        );
        let right: u64 = ic_clk as u64 * (tlow as u64 + tf as u64);
        Self::div_round_closest_ull(right, Self::MICRO) - 1 + offset
    }

    /// DesignWare I2C core doesn't seem to have solid strategy to meet
    /// the tHD;STA timing spec.  Configuring _HCNT based on tHIGH spec
    /// will result in violation of the tHD;STA spec.
    /// Conditional expression1:
    /// IC_[FS]S_SCL_HCNT + (1+4+3) >= IC_CLK * tHIGH
    /// This is based on the DW manuals, and represents an ideal
    /// configuration.  The resulting I2C bus speed will be
    /// If your hardware is free from tHD;STA issue, try this one.
    ///
    /// Conditional expression2:
    /// IC_[FS]S_SCL_HCNT + 3 >= IC_CLK * (tHD;STA + tf)
    /// This is just experimental rule; the tHD;STA period turned
    /// out to be proportinal to (_HCNT + 3).  With this setting
    /// we could meet both tHIGH and tHD;STA timing specs.
    /// If unsure, you'd better to take this alternative.
    ///
    /// The reason why we need to take into account "tf" here,
    /// is the same as described in i2c_dw_scl_lcnt().
    pub(crate) fn scl_hcnt(ic_clk: u32, tsymbol: u32, tf: u32, cond: bool, offset: u32) -> u32 {
        if cond {
            let right: u64 = ic_clk as u64 * tsymbol as u64;
            Self::div_round_closest_ull(right, Self::MICRO) - 8 + offset
        } else {
            let right: u64 = ic_clk as u64 * (tsymbol as u64 + tf as u64);
            Self::div_round_closest_ull(right, Self::MICRO) - 3 + offset
        }
    }
}

bitflags! {
    /// I2C DRIVER STATUS
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub(crate) struct DwI2cStatus: u32 {
         /// Support I2C
         const ACTIVE           = 0x01;
         const WriteInProgress  = 1<<1;
         const ReadInProgress   = 1<<2;
    }
}

bitflags! {
    /// I2C cmd err
    #[repr(transparent)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub(crate) struct DwI2cCmdErr: u32 {
        const TX_ABRT = 0x1;
    }
}
