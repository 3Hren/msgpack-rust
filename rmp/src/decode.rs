use core::fmt::{Debug, Display, Formatter};
use std::{error, fmt};
#[doc(inline)]
#[allow(deprecated)]
use crate::errors::Error;
use crate::Marker;
#[cfg(feature="sync")]
pub use crate::sync::decode::dec::{read_f32, read_f64};
#[cfg(feature="sync")]
pub use crate::sync::decode::ext::{
    read_ext_meta, read_fixext1, read_fixext16, read_fixext2, read_fixext4, read_fixext8, ExtMeta,
};
#[cfg(feature="sync")]
pub use crate::sync::decode::sint::{read_i16, read_i32, read_i64, read_i8, read_nfix};
#[allow(deprecated)]
// While we re-export deprecated items, we don't want to trigger warnings while compiling this crate
#[cfg(feature="sync")]
pub use crate::sync::decode::str::{read_str, read_str_from_slice, read_str_len, read_str_ref, DecodeStringError};
#[cfg(feature="sync")]
pub use crate::sync::decode::uint::{read_pfix, read_u16, read_u32, read_u64, read_u8};

#[cfg(feature = "sync")]
pub use crate::sync::decode::{
    RmpRead, read_array_len, read_bin_len, read_bool, read_marker, read_nil, read_map_len,marker_to_len, read_int,bytes::Bytes,
};


/// An error which can occur when attempting to read a MessagePack numeric value from the reader.
#[derive(Debug)]
#[allow(deprecated)] // Used for compatibility
pub enum NumValueReadError<E: RmpReadErr = Error> {
    /// Failed to read the marker.
    InvalidMarkerRead(E),
    /// Failed to read the data.
    InvalidDataRead(E),
    /// The type decoded isn't match with the expected one.
    TypeMismatch(Marker),
    /// Out of range integral type conversion attempted.
    OutOfRange,
}

/// The error type for I/O operations on `RmpRead` and associated traits.
///
/// For [std::io::Read], this is [std::io::Error]
pub trait RmpReadErr: Display + Debug + crate::errors::MaybeErrBound + 'static {}
#[cfg(feature = "std")]
impl RmpReadErr for std::io::Error {}
impl RmpReadErr for core::convert::Infallible {}

#[cfg(feature = "std")]
impl error::Error for NumValueReadError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            NumValueReadError::InvalidMarkerRead(ref err) |
            NumValueReadError::InvalidDataRead(ref err) => Some(err),
            NumValueReadError::TypeMismatch(..) |
            NumValueReadError::OutOfRange => None,
        }
    }
}

impl<E: RmpReadErr> Display for NumValueReadError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(match *self {
            NumValueReadError::InvalidMarkerRead(..) => "failed to read MessagePack marker",
            NumValueReadError::InvalidDataRead(..) => "failed to read MessagePack data",
            NumValueReadError::TypeMismatch(..) => {
                "the type decoded isn't match with the expected one"
            }
            NumValueReadError::OutOfRange => "out of range integral type conversion attempted",
        })
    }
}

impl<E: RmpReadErr> From<MarkerReadError<E>> for NumValueReadError<E> {
    #[cold]
    fn from(err: MarkerReadError<E>) -> NumValueReadError<E> {
        match err {
            MarkerReadError(err) => NumValueReadError::InvalidMarkerRead(err),
        }
    }
}

impl<E: RmpReadErr> From<ValueReadError<E>> for NumValueReadError<E> {
    #[cold]
    fn from(err: ValueReadError<E>) -> NumValueReadError<E> {
        match err {
            ValueReadError::InvalidMarkerRead(err) => NumValueReadError::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => NumValueReadError::InvalidDataRead(err),
            ValueReadError::TypeMismatch(err) => NumValueReadError::TypeMismatch(err),
        }
    }
}

// An error returned from the `write_marker` and `write_fixval` functions.
struct MarkerWriteError<E: RmpReadErr>(E);

impl<E: RmpReadErr> From<E> for MarkerWriteError<E> {
    #[cold]
    fn from(err: E) -> Self {
        MarkerWriteError(err)
    }
}


/// An error that can occur when attempting to read a MessagePack marker from the reader.
#[derive(Debug)]
#[allow(deprecated)] // Needed for backwards compat
pub struct MarkerReadError<E: RmpReadErr = Error>(pub E);

/// An error which can occur when attempting to read a MessagePack value from the reader.
#[derive(Debug)]
#[allow(deprecated)] // Needed for backwards compat
pub enum ValueReadError<E: RmpReadErr = Error> {
    /// Failed to read the marker.
    InvalidMarkerRead(E),
    /// Failed to read the data.
    InvalidDataRead(E),
    /// The type decoded isn't match with the expected one.
    TypeMismatch(Marker),
}

#[cfg(feature = "std")]
impl error::Error for ValueReadError {
    #[cold]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            ValueReadError::InvalidMarkerRead(ref err) |
            ValueReadError::InvalidDataRead(ref err) => Some(err),
            ValueReadError::TypeMismatch(..) => None,
        }
    }
}

impl Display for ValueReadError {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        // TODO: This should probably use formatting
        f.write_str(match *self {
            ValueReadError::InvalidMarkerRead(..) => "failed to read MessagePack marker",
            ValueReadError::InvalidDataRead(..) => "failed to read MessagePack data",
            ValueReadError::TypeMismatch(..) => {
                "the type decoded isn't match with the expected one"
            }
        })
    }
}

impl<E: RmpReadErr> From<MarkerReadError<E>> for ValueReadError<E> {
    #[cold]
    fn from(err: MarkerReadError<E>) -> ValueReadError<E> {
        match err {
            MarkerReadError(err) => ValueReadError::InvalidMarkerRead(err),
        }
    }
}

impl<E: RmpReadErr> From<E> for MarkerReadError<E> {
    #[cold]
    fn from(err: E) -> MarkerReadError<E> {
        MarkerReadError(err)
    }
}
