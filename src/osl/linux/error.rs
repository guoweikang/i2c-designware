use kernel::prelude::error{Error,code};

impl From<crate::error::Errno> for Error {
    fn from(errno: crate::error::Errno) -> Self {
        match errno {
            crate::error::Errno::InvalidArgs => code::EINVAL,
        }
    }
}

