//! Provides various functions and structs for MessagePack encoding.

pub(crate) mod bin;
pub(crate) mod dec;
pub(crate) mod ext;
pub(crate) mod map;
pub(crate) mod sint;
pub(crate) mod str;
pub(crate) mod uint;
pub(crate) mod vec;


use crate::Marker;

pub mod buffer;

pub use buffer::ByteBuf;
pub use crate::encode::{DataWriteError, ValueWriteError,RmpWriteErr};
use crate::encode::{MarkerWriteError};

#[doc(inline)]
#[allow(deprecated)]
pub use crate::errors::Error;

pub use crate::sync::encode::bin::{write_bin, write_bin_len};
pub use crate::sync::encode::dec::{write_f32, write_f64};
pub use crate::sync::encode::sint::{write_i16, write_i32, write_i64, write_i8, write_nfix, write_sint};
pub use crate::sync::encode::str::{write_str, write_str_len};
pub use crate::sync::encode::uint::{write_pfix, write_u16, write_u32, write_u64, write_u8, write_uint};

/// Attempts to write the given marker into the writer.
fn write_marker<W: RmpWrite>(wr: &mut W, marker: Marker) -> Result<(), MarkerWriteError<W::Error>> {
    wr.write_u8(marker.to_u8()).map_err(MarkerWriteError)
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
/// See also [std::uo::Write] and [byteorder::WriteBytesExt]
///
/// Its primary implementations are [std::io::Write] and [ByteBuf].
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
    /// See also [std::io::Write::write_all]
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


/// Encodes and attempts to write the most efficient array length implementation to the given write,
/// returning the marker used.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data.
pub fn write_array_len<W: RmpWrite>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError<W::Error>> {
    let marker = if len < 16 {
        write_marker(wr, Marker::FixArray(len as u8))?;
        Marker::FixArray(len as u8)
    } else if len <= u16::MAX as u32 {
        write_marker(wr, Marker::Array16)?;
        wr.write_data_u16(len as u16)?;
        Marker::Array16
    } else {
        write_marker(wr, Marker::Array32)?;
        wr.write_data_u32(len)?;
        Marker::Array32
    };

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
        write_marker(wr, Marker::FixMap(len as u8))?;
        Marker::FixMap(len as u8)
    } else if len <= u16::MAX as u32 {
        write_marker(wr, Marker::Map16)?;
        wr.write_data_u16(len as u16)?;
        Marker::Map16
    } else {
        write_marker(wr, Marker::Map32)?;
        wr.write_data_u32(len)?;
        Marker::Map32
    };

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
        1 => {
            write_marker(wr, Marker::FixExt1)?;
            Marker::FixExt1
        }
        2 => {
            write_marker(wr, Marker::FixExt2)?;
            Marker::FixExt2
        }
        4 => {
            write_marker(wr, Marker::FixExt4)?;
            Marker::FixExt4
        }
        8 => {
            write_marker(wr, Marker::FixExt8)?;
            Marker::FixExt8
        }
        16 => {
            write_marker(wr, Marker::FixExt16)?;
            Marker::FixExt16
        }
        len if len < 256 => {
            write_marker(wr, Marker::Ext8)?;
            wr.write_data_u8(len as u8)?;
            Marker::Ext8
        }
        len if len < 65536 => {
            write_marker(wr, Marker::Ext16)?;
            wr.write_data_u16(len as u16)?;
            Marker::Ext16
        }
        len => {
            write_marker(wr, Marker::Ext32)?;
            wr.write_data_u32(len)?;
            Marker::Ext32
        }
    };

    wr.write_data_i8(ty)?;

    Ok(marker)
}
