use std::convert;
use std::io;
use std::result;
use std::num::FromPrimitive;
use std::str::Utf8Error;

use byteorder;

const FIXSTR_SIZE   : u8 = 0x1f;
const FIXARRAY_SIZE : u8 = 0x0f;
const FIXMAP_SIZE   : u8 = 0x0f;

#[derive(Debug, PartialEq)]
pub enum Marker {
    PositiveFixnum(u8),
    NegativeFixnum(i8),
    Null,
    True,
    False,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    FixedString(u8),
    Str8,
    Str16,
    Str32,
    Bin8,
    Bin16,
    Bin32,
    FixedArray(u8),
    Array16,
    Array32,
    FixedMap(u8),
    Map16,
    Map32,
    FixExt1,
    FixExt2,
    FixExt4,
    FixExt8,
    FixExt16,
    Ext8,
    Ext16,
    Ext32,
    Reserved,
}

impl FromPrimitive for Marker {
    fn from_i64(n: i64) -> Option<Marker> {
        FromPrimitive::from_u64(n as u64)
    }

    fn from_u64(n: u64) -> Option<Marker> {
        match n {
            val @ 0x00 ... 0x7f => Some(Marker::PositiveFixnum(val as u8)),
            val @ 0xe0 ... 0xff => Some(Marker::NegativeFixnum(val as i8)),
            val @ 0x80 ... 0x8f => Some(Marker::FixedMap((val as u8) & FIXMAP_SIZE)),
            val @ 0x90 ... 0x9f => Some(Marker::FixedArray((val as u8) & FIXARRAY_SIZE)),
            val @ 0xa0 ... 0xbf => Some(Marker::FixedString((val as u8) & FIXSTR_SIZE)),
            0xc0 => Some(Marker::Null),
            0xc1 => None, // Marked in MessagePack spec as never used.
            0xc2 => Some(Marker::False),
            0xc3 => Some(Marker::True),
            0xc4 => Some(Marker::Bin8),
            0xc5 => Some(Marker::Bin16),
            0xc6 => Some(Marker::Bin32),
            0xc7 => Some(Marker::Ext8),
            0xc8 => Some(Marker::Ext16),
            0xc9 => Some(Marker::Ext32),
            0xca => Some(Marker::F32),
            0xcb => Some(Marker::F64),
            0xcc => Some(Marker::U8),
            0xcd => Some(Marker::U16),
            0xce => Some(Marker::U32),
            0xcf => Some(Marker::U64),
            0xd0 => Some(Marker::I8),
            0xd1 => Some(Marker::I16),
            0xd2 => Some(Marker::I32),
            0xd3 => Some(Marker::I64),
            0xd4 => Some(Marker::FixExt1),
            0xd5 => Some(Marker::FixExt2),
            0xd6 => Some(Marker::FixExt4),
            0xd7 => Some(Marker::FixExt8),
            0xd8 => Some(Marker::FixExt16),
            0xd9 => Some(Marker::Str8),
            0xda => Some(Marker::Str16),
            0xdb => Some(Marker::Str32),
            0xdc => Some(Marker::Array16),
            0xdd => Some(Marker::Array32),
            0xde => Some(Marker::Map16),
            0xdf => Some(Marker::Map32),
            _ => None,
        }
    }
}

/// An error type for reading bytes from the reader.
///
/// This is a thin wrapper over the standard `io::Error` type. Namely, it adds one additional error
/// case: an unexpected EOF.
#[unstable(reason = "remove Debug trait")]
#[derive(Debug)]
pub enum ReadError {
    UnexpectedEOF,
    IO(io::Error),
}

// TODO: This is a hack, because io::Error has PartialEq once.
impl PartialEq for ReadError {
    fn eq(&self, other: &ReadError) -> bool {
        match (self, other) {
            (&ReadError::UnexpectedEOF, &ReadError::UnexpectedEOF) => true,
            (&ReadError::IO(ref lhs),   &ReadError::IO(ref rhs)) => {
                lhs.kind() == rhs.kind()
            }
            _ => false
        }
    }

    fn ne(&self, other: &ReadError) -> bool {
        return !self.eq(other);
    }
}

impl convert::From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError { ReadError::IO(err) }
}

impl convert::From<ReadError> for io::Error {
    fn from(err: ReadError) -> io::Error {
        match err {
            ReadError::IO(err) => err,
            ReadError::UnexpectedEOF => io::Error::new(io::ErrorKind::Other, "unexpected EOF")
        }
    }
}

impl convert::From<byteorder::Error> for ReadError {
    fn from(err: byteorder::Error) -> ReadError {
        match err {
            byteorder::Error::UnexpectedEOF => ReadError::UnexpectedEOF,
            byteorder::Error::Io(err) => ReadError::IO(err),
        }
    }
}

#[unstable(reason = "remove Debug trait; drop MarkerError")]
#[derive(Debug, PartialEq)]
pub enum Error {
    /// Marker type error.
    TypeMismatch(Marker),
    /// IO error while reading marker.
    InvalidMarkerRead(ReadError),
    /// IO error while reading data.
    InvalidDataRead(ReadError),
}

#[unstable(reason = "Core? Shit name!")]
#[derive(Debug, PartialEq)]
pub enum DecodeStringError<'a> {
    Core(Error),
    /// The given buffer is too small to accumulate specified amount of bytes.
    BufferSizeTooSmall(u32),
    InvalidDataCopy(&'a [u8], ReadError),
    InvalidUtf8(&'a [u8], Utf8Error),
}

impl<'a> convert::From<Error> for DecodeStringError<'a> {
    fn from(err: Error) -> DecodeStringError<'a> {
        DecodeStringError::Core(err)
    }
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum Integer {
    /// Every non-negative integer is treated as u64, even if it fits in i64.
    U64(u64),
    /// Every negative integer is treated as i64.
    I64(i64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Integer(Integer),
    String(String),
    Array(Vec<Value>),
}

pub mod decode;
