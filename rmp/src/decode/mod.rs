//! Provides various functions and structs for MessagePack decoding.
//!
//! Most of the function defined in this module will silently handle interruption error (EINTR)
//! received from the given `Read` to be in consistent state with the `Write::write_all` method in
//! the standard library.
//!
//! Any other error would immediately interrupt the parsing process. If your reader can results in
//! I/O error and simultaneously be a recoverable state (for example, when reading from
//! non-blocking socket and it returns EWOULDBLOCK) be sure that you buffer the data externally
//! to avoid data loss (using `BufRead` readers with manual consuming or some other way).

use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::Read;

use byteorder::{self, ReadBytesExt};

use Marker;

/// An error that can occur when attempting to read bytes from the reader.
pub type Error = ::std::io::Error;

/// An error that can occur when attempting to read a MessagePack marker from the reader.
struct MarkerReadError(Error);

/// An error which can occur when attempting to read a MessagePack value from the reader.
#[derive(Debug)]
pub enum ValueReadError {
    /// Failed to read the marker.
    InvalidMarkerRead(Error),
    /// Failed to read the data.
    InvalidDataRead(Error),
    /// The type decoded isn't match with the expected one.
    TypeMismatch(Marker),
}

impl error::Error for ValueReadError {
    fn description(&self) -> &str {
        match *self {
            ValueReadError::InvalidMarkerRead(..) => "failed to read MessagePack marker",
            ValueReadError::InvalidDataRead(..) => "failed to read MessagePack data",
            ValueReadError::TypeMismatch(..) => {
                "the type decoded isn't match with the expected one"
            }
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            ValueReadError::InvalidMarkerRead(ref err) => Some(err),
            ValueReadError::InvalidDataRead(ref err) => Some(err),
            ValueReadError::TypeMismatch(..) => None,
        }
    }
}

impl Display for ValueReadError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(f)
    }
}

impl From<MarkerReadError> for ValueReadError {
    fn from(err: MarkerReadError) -> ValueReadError {
        match err {
            MarkerReadError(err) => ValueReadError::InvalidMarkerRead(err),
        }
    }
}

impl From<Error> for MarkerReadError {
    fn from(err: Error) -> MarkerReadError {
        MarkerReadError(err)
    }
}

/// Attempts to read a single byte from the given reader and to decode it as a MessagePack marker.
fn read_marker<R: Read>(rd: &mut R) -> Result<Marker, MarkerReadError> {
    Ok(Marker::from_u8(try!(rd.read_u8())))
}

/// Attempts to read a single byte from the given reader and to decode it as a nil value.
///
/// According to the MessagePack specification, a nil value is represented as a single `0xc0` byte.
///
/// # Errors
///
/// This function will return `FixedValueReadError` on any I/O error while reading the nil marker,
/// except the EINTR, which is handled internally.
///
/// It also returns `FixedValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_nil<R: Read>(rd: &mut R) -> Result<(), ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::Null => Ok(()),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}
