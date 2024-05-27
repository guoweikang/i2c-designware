use osl::driver::i2c::I2cFuncFlags;

pub(crate) const DW_I2C_DEFAULT_FUNCTIONALITY: I2cFuncFlags = I2cFuncFlags::I2C
    .union(I2cFuncFlags::SMBUS_BYTE)
    .union(I2cFuncFlags::SMBUS_BYTE_DATA)
    .union(I2cFuncFlags::SMBUS_WORD_DATA)
    .union(I2cFuncFlags::SMBUS_BLOCK_DATA)
    .union(I2cFuncFlags::SMBUS_I2C_BLOCK);
