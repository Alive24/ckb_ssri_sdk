use ckb_std::error::SysError;

/// Error
#[repr(i8)]
#[derive(Debug)]
pub enum Error {
    IndexOutOfBound = 1,
    ItemMissing,
    LengthNotEnough,
    Encoding,
    SpawnExceededMaxContentLength,
    SpawnWrongMemoryLimit,
    SpawnExceededMaxPeakMemory,

    InvalidVmVersion,
    InvalidMethodPath,
    InvalidMethodArgs,
}

impl From<SysError> for Error {
    fn from(err: SysError) -> Self {
        use SysError::*;
        match err {
            IndexOutOfBound => Self::IndexOutOfBound,
            ItemMissing => Self::ItemMissing,
            LengthNotEnough(_) => Self::LengthNotEnough,
            Encoding => Self::Encoding,
            SpawnExceededMaxContentLength => Self::SpawnExceededMaxContentLength,
            SpawnWrongMemoryLimit => Self::SpawnWrongMemoryLimit,
            SpawnExceededMaxPeakMemory => Self::SpawnExceededMaxPeakMemory,
            Unknown(err_code) => panic!("unexpected sys error {}", err_code),
        }
    }
}