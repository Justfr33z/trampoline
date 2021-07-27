use std::{result, error, fmt};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ToSmall,
    InvalidTarget,
    Windows(windows::Error)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::ToSmall => write!(f, "value to small"),
            Error::InvalidTarget => write!(f, "invalid target"),
            Error::Windows(ref err) => write!(f, "windows api failed '{}'", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::ToSmall => None,
            Error::InvalidTarget => None,
            Error::Windows(ref err) => Some(err),
        }
    }
}

impl From<windows::Error> for Error {
    fn from(err: windows::Error) -> Self {
        Error::Windows(err)
    }
}

