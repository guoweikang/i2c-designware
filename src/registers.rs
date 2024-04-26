//! The official documentation: <https://www.synopsys.com/dw/ipdir.php?c=DW_apb_i2c>

use core::ptr::NonNull;
use core::ops::Deref;

use tock_registers::register_bitfields;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

/// DwApbI2cRegisters pointer wrapper
pub(crate) struct DwApbI2cRegistersRef {
    ptr: NonNull<DwApbI2cRegisters>,
}

impl DwApbI2cRegistersRef {
    /// Create a new `StaticRef` from a raw pointer
    ///
    /// ## Safety
    ///
    /// - `ptr` must be aligned, non-null, and dereferencable as `T`.
    /// - `*ptr` must be valid for the program duration.
    pub(crate) const fn new(ptr: *mut u8) -> DwApbI2cRegistersRef {
        DwApbI2cRegistersRef {
            ptr: NonNull::new(ptr).expect("ptr os null").cast(),
        }
    }
}

impl Deref for DwApbI2cRegistersRef {
    type Target = DwApbI2cRegisters;

    fn deref(&self) -> &DwApbI2cRegisters {
        // SAFETY: `ptr` is aligned and dereferencable for the program
        // duration as promised by the caller of `StaticRef::new`.
        unsafe { self.ptr.as_ref() }
    }
}

#[repr(C)]
#[allow(non_snake_case)]
pub(crate) struct DwApbI2cRegisters {
    /// Distributor Control Register.
    pub(crate) IC_CON: ReadWrite<u32, IC_CON::Register>,
    pub(crate) IC_TAR: ReadWrite<u32, IC_TAR::Register>,
    pub(crate) IC_SAR: ReadWrite<u32, IC_SAR::Register>,
    pub(crate) IC_HS_MADDR: ReadWrite<u32, IC_HS_MADDR::Register>,
    pub(crate) IC_DATA_CMD: ReadWrite<u32, IC_DATA_CMD::Register>,
    pub(crate) IC_SS_OR_UFM_SCL_HCNT: ReadWrite<u32, IC_GENERAL_CNT::Register>,
    pub(crate) IC_SS_OR_UFM_SCL_LCNT: ReadWrite<u32, IC_GENERAL_CNT::Register>,
    pub(crate) IC_FS_SCL_HCNT_OR_UFM_TBUF_CNT: ReadWrite<u32, IC_GENERAL_CNT::Register>,
    pub(crate) IC_FS_SCL_LCNT: ReadWrite<u32, IC_GENERAL_CNT::Register>,
    pub(crate) IC_HS_SCL_HCNT: ReadWrite<u32, IC_GENERAL_CNT::Register>,
    pub(crate) IC_HS_SCL_LCNT: ReadWrite<u32, IC_GENERAL_CNT::Register>,
    pub(crate) IC_INTR_STAT: ReadOnly<u32, IC_INTR::Register>,
    pub(crate) IC_INTR_MASK: ReadWrite<u32, IC_INTR::Register>,
    pub(crate) IC_RAW_INTR_STAT: ReadOnly<u32, IC_INTR::Register>,
    pub(crate) IC_RX_TL: ReadWrite<u32, IC_RX_TL::Register>,
    pub(crate) IC_TX_TL: ReadWrite<u32, IC_TX_TL::Register>,
    pub(crate) IC_CLR_INTR: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_RX_UNDER: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_RX_OVER: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_TX_OVER: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_RD_REQ: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_TX_ABRT: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_RX_DONE: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_ACTIVITY: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_STOP_DET: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_START_DET: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_CLR_GEN_CALL: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_ENABLE: ReadWrite<u32, IC_ENABLE::Register>,
    pub(crate) IC_STATUS: ReadOnly<u32, IC_STATUS::Register>,
    pub(crate) IC_TXFLR: ReadOnly<u32, IC_GENERAL_FLR::Register>,
    pub(crate) IC_RXFLR: ReadOnly<u32, IC_GENERAL_FLR::Register>,
    pub(crate) IC_SDA_HOLD: ReadWrite<u32, IC_SDA_HOLD::Register>,
    pub(crate) IC_TX_ABRT_SOURCE: ReadOnly<u32, IC_TX_ABRT_SOURCE::Register>,
    pub(crate) IC_SLV_DATA_NACK_ONLY: ReadWrite<u32, IC_SLV_DATA_NACK_ONLY::Register>,
    pub(crate) IC_DMA_CR: ReadWrite<u32, IC_DMA_CR::Register>,
    pub(crate) IC_DMA_TDLR: ReadWrite<u32, IC_GENERAL_FLR::Register>,
    pub(crate) IC_DMA_RDLR: ReadWrite<u32, IC_GENERAL_FLR::Register>,
    pub(crate) IC_SDA_SETUP: ReadWrite<u32, IC_SDA_SETUP::Register>,
    pub(crate) IC_ACK_GENERAL_CALL: ReadWrite<u32, IC_ACK_GENERAL_CALL::Register>,
    pub(crate) IC_ENABLE_STATUS: ReadOnly<u32, IC_ENABLE_STATUS::Register>,

    pub(crate) IC_FS_OR_UFM_SPKLEN: ReadWrite<u32, IC_GENERAL_SPKLEN::Register>,
    pub(crate) IC_HS_SPKLEN: ReadWrite<u32, IC_GENERAL_SPKLEN::Register>,
    pub(crate) IC_CLR_RESTART_DET: ReadOnly<u32, IC_GENERAL_CLR::Register>,

    pub(crate) IC_SCL_STUCK_AT_LOW_TIMEOUT: ReadWrite<u32, IC_GENERAL_TIMEOUT::Register>,
    pub(crate) IC_SDA_STUCK_AT_LOW_TIMEOUT: ReadWrite<u32, IC_GENERAL_TIMEOUT::Register>,
    pub(crate) IC_CLR_SCL_STUCK_DET: ReadOnly<u32, IC_GENERAL_CLR::Register>,
    pub(crate) IC_DEVICE_ID: ReadOnly<u32, IC_DEVICE_ID::Register>,

    pub(crate) IC_SMBUS_CLOCK_LOW_SEXT: ReadWrite<u32, IC_GENERAL_TIMEOUT::Register>,
    pub(crate) IC_SMBUS_CLOCK_LOW_MEXT: ReadWrite<u32, IC_GENERAL_TIMEOUT::Register>,
    pub(crate) IC_SMBUS_THIGH_MAX_IDLE_COUNT: ReadWrite<u32, IC_GENERAL_CNT::Register>,
    pub(crate) IC_SMBUS_INTR_STAT: ReadOnly<u32, IC_SMBUS_INTR::Register>,
    pub(crate) IC_SMBUS_INTR_MASK: ReadWrite<u32, IC_SMBUS_INTR::Register>,
    pub(crate) IC_SMBUS_INTR_RAW_STATUS: ReadOnly<u32, IC_SMBUS_INTR::Register>,
    pub(crate) IC_CLR_SMBUS_INTR: WriteOnly<u32, IC_SMBUS_INTR::Register>,
    pub(crate) IC_OPTIONAL_SAR: ReadWrite<u32, IC_OPTION_SAR::Register>,
    pub(crate) IC_SMBUS_UDID_LSB: ReadWrite<u32, IC_SMBUS_UDID_LSB::Register>,

    _reserved: [u32; 5], // e0-f0
    pub(crate) IC_COMP_PARAM_1: ReadOnly<u32, IC_COMP_PARAM_1::Register>,
    pub(crate) IC_COMP_VERSION: ReadOnly<u32, IC_COMP_VERSION::Register>,
    pub(crate) IC_COMP_TYPE: ReadOnly<u32, IC_COMP_TYPE::Register>,
}

register_bitfields![u32,
     pub(crate)IC_CON [
         SMBUS_PERSISTANT_SLV_ADDR_EN OFFSET(19) NUMBITS(1) [],
         SMBUS_ARP_EN OFFSET(18) NUMBITS(1) [],
         SMBUS_SLAVE_QUICK_CMD_EN OFFSET(17) NUMBITS(1) [],
         OPTIONAL_SAR_CTRL OFFSET(16) NUMBITS(1) [],
         BUS_CLEAR_FEATURE_CTRL OFFSET(11) NUMBITS(1) [],
         STOP_DET_IF_MASTER_ACTIVE OFFSET(10) NUMBITS(1) [],
         RX_FIFO_FULL_HLD_CTRL OFFSET(9) NUMBITS(1) [],
         TX_EMPTY_CTRL OFFSET(8) NUMBITS(1) [],
         STOP_DET_IFADDRESSED OFFSET(7) NUMBITS(1) [],
         IC_SLAVE_DISABLE OFFSET(6) NUMBITS(1) [],
         IC_RESTART_EN OFFSET(5) NUMBITS(1) [],
         IC_10BITADDR_MASTER OFFSET(4) NUMBITS(1) [],
         IC_10BITADDR_SLAVE OFFSET(3) NUMBITS(1) [],
         SPEED OFFSET(1) NUMBITS(2) [],
         MASTER_MODE OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_TAR [
         SMBUS_QUICK_CMD OFFSET(16) NUMBITS(1) [],
         DEVICE_ID OFFSET(13) NUMBITS(1) [],
         IC_10BITADDR_MASTER OFFSET(12) NUMBITS(1) [],
         SPECIAL OFFSET(11) NUMBITS(1) [],
         GC_OR_START OFFSET(10) NUMBITS(1) [],
         TAR OFFSET(0) NUMBITS(10) [],
     ],

     pub(crate) IC_SAR [
         SAR OFFSET(0) NUMBITS(10) [],
     ],

     pub(crate) IC_HS_MADDR [
         HS_MADDR OFFSET(0) NUMBITS(3) [],
     ],

     pub(crate) IC_DATA_CMD [
         FIRST_DATA_BYTE OFFSET(11) NUMBITS(1) [],
         RESTART OFFSET(10) NUMBITS(1) [],
         STOP OFFSET(9) NUMBITS(1) [],
         CMD OFFSET(8) NUMBITS(1) [],
         DAT OFFSET(0) NUMBITS(8) [],
     ],

     pub(crate) IC_GENERAL_CNT [
         CNT OFFSET(0) NUMBITS(16) [],
     ],

     pub(crate) IC_INTR [
         SCL_STUCK_AT_LOW OFFSET(14) NUMBITS(1) [],
         MST_ON_HOLD OFFSET(13) NUMBITS(1) [],
         RESTART_DET OFFSET(12) NUMBITS(1) [],
         GEN_CALL OFFSET(11) NUMBITS(1) [],
         START_DET OFFSET(10) NUMBITS(1) [],
         STOP_DET OFFSET(9) NUMBITS(1) [],
         ACTIVITY OFFSET(8) NUMBITS(1) [],
         RX_DONE OFFSET(7) NUMBITS(1) [],
         TX_ABRT OFFSET(6) NUMBITS(1) [],
         RD_REQ OFFSET(5) NUMBITS(1) [],
         TX_EMPTY OFFSET(4) NUMBITS(1) [],
         TX_OVER OFFSET(3) NUMBITS(1) [],
         RX_FULL OFFSET(2) NUMBITS(1) [],
         RX_OVER OFFSET(1) NUMBITS(1) [],
         RX_UNDER OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_RX_TL [
         RX_TL OFFSET(0) NUMBITS(8) [],
     ],

     pub(crate) IC_TX_TL [
         TX_TL OFFSET(0) NUMBITS(8) [],
     ],

     pub(crate) IC_GENERAL_CLR [
         CLR OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_ENABLE [
         SMBUS_ALERT_EN OFFSET(18) NUMBITS(1) [],
         SMBUS_SUSPEND_EN OFFSET(17) NUMBITS(1) [],
         SMBUS_CLK_RESET OFFSET(16) NUMBITS(1) [],
         SDA_STUCK_RECOVERY_ENABLE OFFSET(3) NUMBITS(1) [],
         TX_CMD_BLOCK OFFSET(2) NUMBITS(1) [],
         ABORT OFFSET(1) NUMBITS(1) [],
         ENABLE OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_STATUS [
         SMBUS_ALERT_STATUS OFFSET(20) NUMBITS(1) [],
         SMBUS_SUSPEND_STATUS OFFSET(19) NUMBITS(1) [],
         SMBUS_SLAVE_ADDR_RESOLVED OFFSET(18) NUMBITS(1) [],
         SMBUS_SLAVE_ADDR_VALID OFFSET(17) NUMBITS(1) [],
         SMBUS_QUICK_CMD_BIT OFFSET(16) NUMBITS(1) [],
         SDA_STUCK_NOT_RECOVERED OFFSET(11) NUMBITS(1) [],
         SLV_HOLD_RX_FIFO_FULL OFFSET(10) NUMBITS(1) [],
         SLV_HOLD_TX_FIFO_EMPTY OFFSET(9) NUMBITS(1) [],
         MST_HOLD_RX_FIFO_FULL OFFSET(8) NUMBITS(1) [],
         MST_HOLD_TX_FIFO_EMPTY OFFSET(7) NUMBITS(1) [],
         SLV_ACTIVITY OFFSET(6) NUMBITS(1) [],
         MST_ACTIVITY OFFSET(5) NUMBITS(1) [],
         RFF OFFSET(4) NUMBITS(1) [],
         RFNE OFFSET(3) NUMBITS(1) [],
         TFE OFFSET(2) NUMBITS(1) [],
         TFNF OFFSET(1) NUMBITS(1) [],
         ACTIVITY OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_GENERAL_FLR [
         CNT OFFSET(0) NUMBITS(32) [],
     ],

     pub(crate) IC_SDA_HOLD [
         SDA_RX_HOLD OFFSET(16) NUMBITS(8) [],
         SDA_TX_HOLD OFFSET(0) NUMBITS(16) [],
     ],


     pub(crate) IC_TX_ABRT_SOURCE [
        TX_FLUSH_CNT OFFSET(23) NUMBITS(9) [],
        ABRT_DEVICE_WRITE OFFSET(20) NUMBITS(1) [],
        ABRT_DEVICE_SLVADDR_NOACK OFFSET(19) NUMBITS(1) [],
        ABRT_DEVICE_NOACK OFFSET(18) NUMBITS(1) [],
        ABRT_SDA_STUCK_AT_LOW OFFSET(17) NUMBITS(1) [],
        ABRT_USER_ABRT OFFSET(16) NUMBITS(1) [],
        ABRT_SLVRD_INTX OFFSET(15) NUMBITS(1) [],
        ABRT_SLV_ARBLOST OFFSET(14) NUMBITS(1) [],
        ABRT_SLVFLUSH_TXFIFO OFFSET(13) NUMBITS(1) [],
        ARB_LOST OFFSET(12) NUMBITS(1) [],
        ABRT_MASTER_DIS OFFSET(11) NUMBITS(1) [],
        ABRT_10B_RD_NORSTRT OFFSET(10) NUMBITS(1) [],
        ABRT_SBYTE_NORSTRT OFFSET(9) NUMBITS(1) [],
        ABRT_HS_NORSTRT OFFSET(8) NUMBITS(1) [],
        ABRT_SBYTE_ACKDET OFFSET(7) NUMBITS(1) [],
        ABRT_HS_ACKDET OFFSET(6) NUMBITS(1) [],
        ABRT_GCALL_READ OFFSET(5) NUMBITS(1) [],
        ABRT_GCALL_NOACK  OFFSET(4) NUMBITS(1) [],
        ABRT_TXDATA_NOACK  OFFSET(3) NUMBITS(1) [],
        ABRT_10ADDR2_NOACK OFFSET(2) NUMBITS(1) [],
        ABRT_10ADDR1_NOACK  OFFSET(1) NUMBITS(1) [],
        ABRT_7B_ADDR_NOACK  OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_SLV_DATA_NACK_ONLY [
        NACK OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_DMA_CR [
        TDMAE OFFSET(1) NUMBITS(1) [],
        RDMAE OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_SDA_SETUP [
        SDA_SETUP OFFSET(0) NUMBITS(8) [],
     ],

     pub(crate) IC_ACK_GENERAL_CALL [
        ACK_GENERAL_CALL OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_ENABLE_STATUS [
        SLV_RX_DATA_LOST OFFSET(2) NUMBITS(1) [],
        SLV_DISABLED_WHILE_BUSY OFFSET(1) NUMBITS(1) [],
        IC_EN OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_GENERAL_SPKLEN [
        SPKLEN OFFSET(0) NUMBITS(8) [],
     ],

     pub(crate) IC_COMP_PARAM_1 [
        TX_BUFFER_DEPTH OFFSET(16) NUMBITS(8) [],
        RX_BUFFER_DEPTH OFFSET(8) NUMBITS(8) [],
        ADD_ENCODED_PARAMS OFFSET(7) NUMBITS(1) [],
        HAS_DMA OFFSET(6) NUMBITS(1) [],
        INTR_IO OFFSET(5) NUMBITS(1) [],
        HC_COUNT_VALUES OFFSET(4) NUMBITS(1) [],
        MAX_SPEED_MODE OFFSET(2) NUMBITS(2) [],
        APB_DATA_WIDTH OFFSET(0) NUMBITS(2) [],
     ],

     pub(crate) IC_COMP_VERSION [
        VERSION OFFSET(0) NUMBITS(32) [],
     ],

     pub(crate) IC_COMP_TYPE [
        TYPE OFFSET(0) NUMBITS(32) [],
     ],

     pub(crate) IC_GENERAL_TIMEOUT [
        TIMEOUT OFFSET(0) NUMBITS(32) [],
     ],

     pub(crate) IC_DEVICE_ID [
        DEVICE_ID OFFSET(0) NUMBITS(24) [],
     ],

     pub(crate) IC_SMBUS_INTR [
        SMBUS_ALERT_DET OFFSET(10) NUMBITS(1) [],
        SMBUS_SUSPEND_DET OFFSET(9) NUMBITS(1) [],
        SLV_RX_PEC_NACK OFFSET(8) NUMBITS(1) [],
        ARP_ASSGN_ADDR_CMD_DET OFFSET(7) NUMBITS(1) [],
        ARP_GET_UDUD_CMD_DET OFFSET(6) NUMBITS(1) [],
        APR_RST_CMD_DET OFFSET(5) NUMBITS(1) [],
        ARP_PREPARE_CMD_DET OFFSET(4) NUMBITS(1) [],
        HOST_NOTIFY_MST_DET OFFSET(3) NUMBITS(1) [],
        QUICK_CMD_DET OFFSET(1) NUMBITS(1) [],
        MST_CLOCK_EXTND_TIMEOUT OFFSET(1) NUMBITS(1) [],
        SLV_CLOCK_EXTND_TIMEOUT OFFSET(0) NUMBITS(1) [],
     ],

     pub(crate) IC_OPTION_SAR [
        OPTION_SAR OFFSET(0) NUMBITS(7) [],
     ],

     pub(crate) IC_SMBUS_UDID_LSB [
        ARP_UDID OFFSET(0) NUMBITS(32) [],
     ],
];

/// Designware Component Type number = 0x44_57_01_40. This 
/// assigned unique hex value is constant and is derived from the two
/// ASCII letters “DW” followed by a 16-bit unsigned number.
pub(crate) const DW_IC_COMP_TYPE_VALUE:u32 = 0x44570140;
