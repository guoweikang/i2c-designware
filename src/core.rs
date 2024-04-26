use bitflags::bitflags;

use crate::common::functionality::*;

pub(crate) const DW_I2C_DEFAULT_FUNCTIONALITY: I2cFuncFlags = I2cFuncFlags::I2C.union(I2cFuncFlags::SMBUS_BYTE)
                                                   .union(I2cFuncFlags::SMBUS_BYTE_DATA) 
                                                   .union(I2cFuncFlags::SMBUS_WORD_DATA) 
                                                   .union(I2cFuncFlags::SMBUS_BLOCK_DATA) 
                                                   .union(I2cFuncFlags::SMBUS_I2C_BLOCK);

bitflags! {
    /// To determine what I2C functionality is present
    #[allow(non_camel_case_types)]
    #[repr(transparent)]
    #[derive(Debug, PartialEq, Eq)]
    pub struct DwI2cConfigFlags: u32 {
         /// Is master
         const MASTER             = 0x00000001;
         /// Speed Std
         const SPEED_STD          = 1<<1;
         /// Speed Fast
         const SPEED_FAST         = 1<<2;
         /// 10BITADDR_SLAVE
         const TEN_BITADDR_SLAVE  = 1<<3;
         /// 10BITADDR_MASTER
         const TEN_BITADDR_MASTER  = 1<<4;
         /// Enabel Restart
         const RESTART_EN  = 1<<5;
         /// Disable Slave
         const SLAVE_DISABLE  = 1<<6;
         /// STOP_DET_IFADDRESSED
         const STOP_DET_IFADDRESSED  = 1<<7;
         /// TX_EMPTY_CTRL
         const TX_EMPTY_CTRL  = 1<<8;
         /// RX_FIFO_FULL_HLD_CTRL
         const RX_FIFO_FULL_HLD_CTRL  = 1<<9;
         /// BUS_CLEAR_CTRL
         const BUS_CLEAR_CTRL  = 1<<11;


         // multi bits
         /// Speed High
         const SPEED_HIGH         = Self::SPEED_STD.bits() | Self::SPEED_FAST.bits();
    }
}
