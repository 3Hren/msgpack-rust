use core::fmt::{Debug, Display, Formatter};
use std::fmt;
#[cfg(feature = "sync")]
pub use crate::sync::encode::bin::{write_bin, write_bin_len};
#[cfg(feature = "sync")]
pub use crate::sync::encode::dec::{write_f32, write_f64};
#[cfg(feature = "sync")]
pub use crate::sync::encode::sint::{write_i16, write_i32, write_i64, write_i8, write_nfix, write_sint};
#[cfg(feature = "sync")]
pub use crate::sync::encode::str::{write_str, write_str_len};
#[cfg(feature = "sync")]
pub use crate::sync::encode::uint::{write_pfix, write_u16, write_u32, write_u64, write_u8, write_uint};
#[cfg(feature = "sync")]
pub use crate::sync::encode::{
    RmpWrite, write_nil,write_bool,write_array_len,write_map_len,write_ext_meta, Error, buffer::ByteBuf,
};

/// The error type for operations on the [RmpWrite] trait.
///
/// For [std::io::Write], this is [std::io::Error]
/// For [ByteBuf], this is [core::convert::Infallible]
pub trait RmpWriteErr: Display + Debug + crate::errors::MaybeErrBound + 'static {}

#[cfg(feature = "std")]
impl RmpWriteErr for std::io::Error {}

impl RmpWriteErr for core::convert::Infallible {}

// An error returned from the `write_marker` and `write_fixval` functions.
pub(crate) struct MarkerWriteError<E: RmpWriteErr>(pub(crate) E);

impl<E: RmpWriteErr> From<E> for MarkerWriteError<E> {
    #[cold]
    fn from(err: E) -> Self {
        MarkerWriteError(err)
    }
}


/// An error returned from primitive values write functions.
#[doc(hidden)]
pub struct DataWriteError<E: RmpWriteErr>(pub(crate) E);

impl<E: RmpWriteErr> From<E> for DataWriteError<E> {
    #[cold]
    #[inline]
    fn from(err: E) -> DataWriteError<E> {
        DataWriteError(err)
    }
}

/// An error that can occur when attempting to write multi-byte MessagePack value.
#[derive(Debug)]
#[allow(deprecated)] // TODO: Needed for compatibility
pub enum ValueWriteError<E: RmpWriteErr = Error> {
    /// I/O error while writing marker.
    InvalidMarkerWrite(E),
    /// I/O error while writing data.
    InvalidDataWrite(E),
}
#[cfg(feature = "std")]
impl From<std::io::Error> for ValueWriteError {
    #[inline]
    fn from(err: std::io::Error) -> Self {
        ValueWriteError::InvalidDataWrite(err)
    }
}
impl<E: RmpWriteErr> From<MarkerWriteError<E>> for ValueWriteError<E> {
    #[cold]
    fn from(err: MarkerWriteError<E>) -> Self {
        match err {
            MarkerWriteError(err) => ValueWriteError::InvalidMarkerWrite(err),
        }
    }
}

impl<E: RmpWriteErr> From<DataWriteError<E>> for ValueWriteError<E> {
    #[cold]
    fn from(err: DataWriteError<E>) -> Self {
        match err {
            DataWriteError(err) => ValueWriteError::InvalidDataWrite(err),
        }
    }
}

#[cfg(feature = "std")] // Backwards compatbility ;)
impl From<ValueWriteError<std::io::Error>> for std::io::Error {
    #[cold]
    fn from(err: ValueWriteError<std::io::Error>) -> std::io::Error {
        match err {
            ValueWriteError::InvalidMarkerWrite(err) |
            ValueWriteError::InvalidDataWrite(err) => err,
        }
    }
}

#[cfg(feature = "std")]
impl<E: RmpWriteErr> std::error::Error for ValueWriteError<E> {
    #[cold]
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ValueWriteError::InvalidMarkerWrite(ref err) |
            ValueWriteError::InvalidDataWrite(ref err) => Some(err),
        }
    }
}

impl<E: RmpWriteErr> Display for ValueWriteError<E> {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("error while writing multi-byte MessagePack value")
    }
}