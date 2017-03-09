use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, ErrorKind};

use rmp::decode::{MarkerReadError, ValueReadError};

pub mod value;
pub mod value_ref;

pub use self::value::read_value;
pub use self::value_ref::read_value_ref;

/// This type represents all possible errors that can occur when deserializing a value.
#[derive(Debug)]
pub enum Error {
    /// Error while reading marker byte.
    InvalidMarkerRead(io::Error),
    /// Error while reading data.
    InvalidDataRead(io::Error),
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match *self {
            Error::InvalidMarkerRead(ref err) => err.kind(),
            Error::InvalidDataRead(ref err) => err.kind(),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidMarkerRead(..) => "I/O error while reading marker byte",
            Error::InvalidDataRead(..) => "I/O error while reading non-marker bytes",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidMarkerRead(ref err) => Some(err),
            Error::InvalidDataRead(ref err) => Some(err),
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match *self {
            Error::InvalidMarkerRead(ref err) => {
                write!(fmt, "I/O error while reading marker byte: {}", err)
            }
            Error::InvalidDataRead(ref err) => {
                write!(fmt, "I/O error while reading non-marker bytes: {}", err)
            }
        }
    }
}

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        Error::InvalidMarkerRead(err.0)
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Error {
        match err {
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            ValueReadError::TypeMismatch(..) => {
                Error::InvalidMarkerRead(io::Error::new(ErrorKind::Other, "type mismatch"))
            }
        }
    }
}
