//! Provides various functions and structs for MessagePack encoding.

mod bin;
mod dec;
mod ext;
mod map;
mod sint;
mod str;
mod uint;
mod vec;

pub use self::bin::{write_bin, write_bin_len};
pub use self::dec::{write_f32, write_f64};
pub use self::sint::{write_i16, write_i32, write_i64, write_i8, write_nfix, write_sint};
pub use self::str::{write_str, write_str_len};
pub use self::uint::{write_pfix, write_u16, write_u32, write_u64, write_u8, write_uint, write_uint8};

use core::fmt::{self, Debug, Display, Formatter};
#[cfg(feature = "std")]
use std::error;

use crate::{Marker, Timestamp};

pub mod buffer;
pub use buffer::ByteBuf;

#[doc(inline)]
#[allow(deprecated)]
pub use crate::errors::Error;

/// The error type for operations on the [`RmpWrite`] trait.
///
/// For [`std::io::Write`], this is [`std::io::Error`]
/// For [`ByteBuf`], this is [`core::convert::Infallible`]
pub trait RmpWriteErr: Display + Debug + crate::errors::MaybeErrBound + 'static {}
#[cfg(feature = "std")]
impl RmpWriteErr for std::io::Error {}
impl RmpWriteErr for core::convert::Infallible {}

// An error returned from the `write_marker` and `write_fixval` functions.
struct MarkerWriteError<E: RmpWriteErr>(E);

impl<E: RmpWriteErr> From<E> for MarkerWriteError<E> {
    #[cold]
    fn from(err: E) -> Self {
        Self(err)
    }
}

/// Attempts to write the given marker into the writer.
fn write_marker<W: RmpWrite>(wr: &mut W, marker: Marker) -> Result<(), MarkerWriteError<W::Error>> {
    wr.write_u8(marker.to_u8()).map_err(MarkerWriteError)
}

/// An error returned from primitive values write functions.
#[doc(hidden)]
pub struct DataWriteError<E: RmpWriteErr>(E);

impl<E: RmpWriteErr> From<E> for DataWriteError<E> {
    #[cold]
    #[inline]
    fn from(err: E) -> Self {
        Self(err)
    }
}

/// Encodes and attempts to write a nil value into the given write.
///
/// According to the MessagePack specification, a nil value is represented as a single `0xc0` byte.
///
/// # Errors
///
/// This function will return `Error` on any I/O error occurred while writing the nil marker.
///
/// # Examples
///
/// ```
/// let mut buf = Vec::new();
///
/// rmp::encode::write_nil(&mut buf).unwrap();
///
/// assert_eq!(vec![0xc0], buf);
/// ```
#[inline]
pub fn write_nil<W: RmpWrite>(wr: &mut W) -> Result<(), W::Error> {
    write_marker(wr, Marker::Null).map_err(|e| e.0)
}

/// Encodes and attempts to write a timestamp value into the given write.
///
/// According to the MessagePack specification, a timestamp value is represented as a 32, 64, or 96 bit Extension struct.
///
/// # Errors
///
/// This function will return `Error` on any I/O error occurred while writing the timestamp.
///
/// # Examples
///
/// ```
/// use rmp::Timestamp;
///
/// let mut buf1 = Vec::new();
/// let mut buf2 = Vec::new();
/// let mut buf3 = Vec::new();
///
/// let ts1 = Timestamp::from_32(0x66c1de7c);
/// let ts2 = Timestamp::from_64(0x66c1de7c, 0x3b9ac9ff).unwrap();
/// let ts3 = Timestamp::from_96(0x66c1de7c, 0x3b9ac9ff).unwrap();
///
/// rmp::encode::write_timestamp(&mut buf1, ts1).ok();
/// rmp::encode::write_timestamp(&mut buf2, ts2).ok();
/// rmp::encode::write_timestamp(&mut buf3, ts3).ok();
///
/// // FixExt4 with a type of -1 (0xff)
/// assert_eq!(vec![0xd6, 0xff, 0x66, 0xc1, 0xde, 0x7c], buf1);
/// // FixExt8 with a type of -1 (0xff)
/// assert_eq!(vec![0xd7, 0xff, 0xee, 0x6b, 0x27, 0xfc, 0x66, 0xc1, 0xde, 0x7c], buf2);
/// // Ext8 with a size of 12 (0x0c) and a type of -1 (0xff)
/// assert_eq!(vec![0xc7, 0x0c, 0xff, 0x3b, 0x9a, 0xc9, 0xff, 0x00, 0x00, 0x00, 0x00, 0x66, 0xc1, 0xde, 0x7c], buf3);
/// ```
#[inline]
pub fn write_timestamp<W: RmpWrite>(wr: &mut W, timestamp: Timestamp) -> Result<(), DataWriteError<W::Error>> {
    match timestamp.size {
        32 => {
            write_marker(wr, Marker::FixExt4).map_err(|e| e.0)?;
            wr.write_data_i8(-1)?;
            wr.write_data_u32(timestamp.secs as u32)?;
        },
        64 => {
            write_marker(wr, Marker::FixExt8).map_err(|e| e.0)?;
            let data = ((timestamp.nsecs as u64) << 34) | (timestamp.secs as u64);
            wr.write_data_i8(-1)?;
            wr.write_data_u64(data)?;
        },
        96 => {
            write_marker(wr, Marker::Ext8).map_err(|e| e.0)?;
            wr.write_data_u8(12)?;
            wr.write_data_i8(-1)?;
            wr.write_data_u32(timestamp.nsecs)?;
            wr.write_data_i64(timestamp.secs)?;
        },
        _ => unreachable!(),
    }
    Ok(())
}

/// Encodes and attempts to write a bool value into the given write.
///
/// According to the MessagePack specification, an encoded boolean value is represented as a single
/// byte.
///
/// # Errors
///
/// Each call to this function may generate an I/O error indicating that the operation could not be
/// completed.
#[inline]
pub fn write_bool<W: RmpWrite>(wr: &mut W, val: bool) -> Result<(), W::Error> {
    let marker = if val { Marker::True } else { Marker::False };

    write_marker(wr, marker).map_err(|e| e.0)
}

mod sealed {
    pub trait Sealed {}
    #[cfg(feature = "std")]
    impl<T: ?Sized + std::io::Write> Sealed for T {}
    #[cfg(not(feature = "std"))]
    impl Sealed for &mut [u8] {}
    #[cfg(not(feature = "std"))]
    impl Sealed for alloc::vec::Vec<u8> {}
    impl Sealed for super::ByteBuf {}
}

macro_rules! write_byteorder_utils {
    ($($name:ident => $tp:ident),* $(,)?) => {
        $(
            #[inline]
            #[doc(hidden)]
            fn $name(&mut self, val: $tp) -> Result<(), DataWriteError<Self::Error>> where Self: Sized {
                const SIZE: usize = core::mem::size_of::<$tp>();
                let mut buf: [u8; SIZE] = [0u8; SIZE];
                paste::paste! {
                    <byteorder::BigEndian as byteorder::ByteOrder>::[<write_ $tp>](&mut buf, val);
                }
                self.write_bytes(&buf).map_err(DataWriteError)
            }
        )*
    };
}

/// A type that `rmp` supports writing into.
///
/// The methods of this trait should be considered an implementation detail (for now).
/// It is currently sealed (can not be implemented by the user).
///
/// See also [`std::io::Write`] and [`byteorder::WriteBytesExt`]
///
/// Its primary implementations are [`std::io::Write`] and [`ByteBuf`].
pub trait RmpWrite: sealed::Sealed {
    type Error: RmpWriteErr;

    /// Write a single byte to this stream
    #[inline]
    fn write_u8(&mut self, val: u8) -> Result<(), Self::Error> {
        let buf = [val];
        self.write_bytes(&buf)
    }

    /// Write a slice of bytes to the underlying stream
    ///
    /// This will either write all the bytes or return an error.
    /// See also [`std::io::Write::write_all`]
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Self::Error>;

    // Internal helper functions to map I/O error into the `DataWriteError` error.

    /// Write a single (signed) byte to this stream.
    #[inline]
    #[doc(hidden)]
    fn write_data_u8(&mut self, val: u8) -> Result<(), DataWriteError<Self::Error>> {
        self.write_u8(val).map_err(DataWriteError)
    }
    /// Write a single (signed) byte to this stream.
    #[inline]
    #[doc(hidden)]
    fn write_data_i8(&mut self, val: i8) -> Result<(), DataWriteError<Self::Error>> {
        self.write_data_u8(val as u8)
    }

    write_byteorder_utils!(
        write_data_u16 => u16,
        write_data_u32 => u32,
        write_data_u64 => u64,
        write_data_i16 => i16,
        write_data_i32 => i32,
        write_data_i64 => i64,
        write_data_f32 => f32,
        write_data_f64 => f64
    );
}

#[cfg(feature = "std")]
impl<T: std::io::Write> RmpWrite for T {
    type Error = std::io::Error;

    #[inline]
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.write_all(buf)
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

impl<E: RmpWriteErr> From<MarkerWriteError<E>> for ValueWriteError<E> {
    #[cold]
    fn from(err: MarkerWriteError<E>) -> Self {
        match err {
            MarkerWriteError(err) => Self::InvalidMarkerWrite(err),
        }
    }
}

impl<E: RmpWriteErr> From<DataWriteError<E>> for ValueWriteError<E> {
    #[cold]
    fn from(err: DataWriteError<E>) -> Self {
        match err {
            DataWriteError(err) => Self::InvalidDataWrite(err),
        }
    }
}

#[cfg(feature = "std")] // Backwards compatbility ;)
impl From<ValueWriteError<Self>> for std::io::Error {
    #[cold]
    fn from(err: ValueWriteError<Self>) -> Self {
        match err {
            ValueWriteError::InvalidMarkerWrite(err) |
            ValueWriteError::InvalidDataWrite(err) => err,
        }
    }
}

#[cfg(feature = "std")]
impl<E: RmpWriteErr> error::Error for ValueWriteError<E> {
    #[cold]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Self::InvalidMarkerWrite(ref err) |
            Self::InvalidDataWrite(ref err) => Some(err),
        }
    }
}

impl<E: RmpWriteErr> Display for ValueWriteError<E> {
    #[cold]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str("error while writing multi-byte MessagePack value")
    }
}

/// Encodes and attempts to write the most efficient array length implementation to the given write,
/// returning the marker used.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data.
pub fn write_array_len<W: RmpWrite>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError<W::Error>> {
    let marker = if len < 16 {
        Marker::FixArray(len as u8)
    } else if u16::try_from(len).is_ok() {
        Marker::Array16
    } else {
        Marker::Array32
    };

    write_marker(wr, marker)?;
    if marker == Marker::Array16 {
        wr.write_data_u16(len as u16)?;
    } else if marker == Marker::Array32 {
        wr.write_data_u32(len)?;
    }
    Ok(marker)
}

/// Encodes and attempts to write the most efficient map length implementation to the given write,
/// returning the marker used.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data.
pub fn write_map_len<W: RmpWrite>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError<W::Error>> {
    let marker = if len < 16 {
        Marker::FixMap(len as u8)
    } else if u16::try_from(len).is_ok() {
        Marker::Map16
    } else {
        Marker::Map32
    };

    write_marker(wr, marker)?;
    if marker == Marker::Map16 {
        wr.write_data_u16(len as u16)?;
    } else if marker == Marker::Map32 {
        wr.write_data_u32(len)?;
    }
    Ok(marker)
}

/// Encodes and attempts to write the most efficient ext metadata implementation to the given
/// write, returning the marker used.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data.
///
/// # Panics
///
/// Panics if `ty` is negative, because it is reserved for future MessagePack extension including
/// 2-byte type information.
pub fn write_ext_meta<W: RmpWrite>(wr: &mut W, len: u32, ty: i8) -> Result<Marker, ValueWriteError<W::Error>> {
    let marker = match len {
        1 => Marker::FixExt1,
        2 => Marker::FixExt2,
        4 => Marker::FixExt4,
        8 => Marker::FixExt8,
        16 => Marker::FixExt16,
        0..=255 => Marker::Ext8,
        256..=65535 => Marker::Ext16,
        _ => Marker::Ext32,
    };
    write_marker(wr, marker)?;

    if marker == Marker::Ext8 {
        wr.write_data_u8(len as u8)?;
    } else if marker == Marker::Ext16 {
        wr.write_data_u16(len as u16)?;
    } else if marker == Marker::Ext32 {
        wr.write_data_u32(len)?;
    }

    wr.write_data_i8(ty)?;

    Ok(marker)
}
