pub mod value {

use std::fmt;
use std::io::Read;
use std::result::Result;
use std::str::Utf8Error;

use super::super::Marker;
pub use super::super::value::{
    Integer,
    Float,
    Value,
};

use super::{
    ReadError,
    MarkerReadError,
    ValueReadError,
    DecodeStringError,
    read_marker,
    read_numeric_data,
    read_str_data,
    read_full,
};

#[derive(Debug)]
pub enum Error {
    InvalidMarkerRead(ReadError),
    InvalidDataRead(ReadError),
    TypeMismatch(Marker),

    BufferSizeTooSmall(u32),
    InvalidDataCopy(Vec<u8>, ReadError),
    InvalidUtf8(Vec<u8>, Utf8Error),

    InvalidArrayRead(Box<Error>),
    InvalidMapKeyRead(Box<Error>),
    InvalidMapValueRead(Box<Error>),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "error while decoding value" }

    fn cause(&self) -> Option<&::std::error::Error> {
        use self::Error::*;
        match *self {
            InvalidMarkerRead(ref err) => Some(err),
            InvalidDataRead(ref err) => Some(err),
            TypeMismatch(_) => None,

            BufferSizeTooSmall(_) => None,
            InvalidDataCopy(_, ref err) => Some(err),
            InvalidUtf8(_, ref err) => Some(err),

            InvalidArrayRead(ref err) => Some(&**err),
            InvalidMapKeyRead(ref err) => Some(&**err),
            InvalidMapValueRead(ref err) => Some(&**err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}


impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        Error::InvalidMarkerRead(From::from(err))
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Error {
        match err {
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            ValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
    }
}

impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError<'a>) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            DecodeStringError::TypeMismatch(marker) => Error::TypeMismatch(marker),
            DecodeStringError::BufferSizeTooSmall(len) => Error::BufferSizeTooSmall(len),
            DecodeStringError::InvalidDataCopy(buf, err) => Error::InvalidDataCopy(buf.to_vec(), err),
            DecodeStringError::InvalidUtf8(buf, err) => Error::InvalidUtf8(buf.to_vec(), err),
        }
    }
}

fn read_str<R>(rd: &mut R, len: u32) -> Result<String, Error>
    where R: Read
{
    let mut vec: Vec<u8> = (0..len).map(|_| 0u8).collect();
    let mut buf = &mut vec[..];
    let data = try!(read_str_data(rd, len, buf));

    Ok(data.to_string())
}

fn read_array<R>(rd: &mut R, len: usize) -> Result<Vec<Value>, Error>
    where R: Read
{
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
        match read_value(rd) {
            Ok(val)  => vec.push(val),
            Err(err) => return Err(Error::InvalidArrayRead(Box::new(err))),
        }
    }

    Ok(vec)
}

fn read_map<R>(rd: &mut R, len: usize) -> Result<Vec<(Value, Value)>, Error>
    where R: Read
{
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
        let key = match read_value(rd) {
            Ok(val)  => val,
            Err(err) => return Err(Error::InvalidMapKeyRead(Box::new(err))),
        };

        let value = match read_value(rd) {
            Ok(val)  => val,
            Err(err) => return Err(Error::InvalidMapValueRead(Box::new(err))),
        };

        vec.push((key, value));
    }

    Ok(vec)
}

fn read_bin_data<R>(rd: &mut R, len: usize) -> Result<Vec<u8>, Error>
    where R: Read
{
    let mut vec: Vec<u8> = (0..len).map(|_| 0u8).collect();

    match read_full(rd, &mut vec[..]) {
        Ok(n) if n == vec.len() => Ok(vec),
        Ok(..)   => Err(Error::InvalidDataRead(ReadError::UnexpectedEOF)),
        Err(err) => Err(Error::InvalidDataRead(ReadError::Io(err))),
    }
}

fn read_ext_body<R>(rd: &mut R, len: usize) -> Result<(i8, Vec<u8>), Error>
    where R: Read
{
    let ty = try!(read_numeric_data(rd));
    let vec = try!(read_bin_data(rd, len));
    Ok((ty, vec))
}

/// Attempts to read bytes from the given reader and interpret them as a `Value`.
///
/// # Errors
///
/// This function will return `Error` on any I/O error while either reading or decoding a `Value`.
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
pub fn read_value<R>(rd: &mut R) -> Result<Value, Error>
    where R: Read
{
    // TODO: Looks creepy.
    let val = match try!(read_marker(rd)) {
        Marker::Null  => Value::Nil,
        Marker::True  => Value::Boolean(true),
        Marker::False => Value::Boolean(false),
        Marker::FixPos(val) => Value::Integer(Integer::U64(val as u64)),
        Marker::FixNeg(val) => Value::Integer(Integer::I64(val as i64)),
        Marker::U8  => Value::Integer(Integer::U64(try!(read_numeric_data::<R, u8>(rd))  as u64)),
        Marker::U16 => Value::Integer(Integer::U64(try!(read_numeric_data::<R, u16>(rd)) as u64)),
        Marker::U32 => Value::Integer(Integer::U64(try!(read_numeric_data::<R, u32>(rd)) as u64)),
        Marker::U64 => Value::Integer(Integer::U64(try!(read_numeric_data(rd)))),
        Marker::I8  => Value::Integer(Integer::I64(try!(read_numeric_data::<R, i8>(rd))  as i64)),
        Marker::I16 => Value::Integer(Integer::I64(try!(read_numeric_data::<R, i16>(rd)) as i64)),
        Marker::I32 => Value::Integer(Integer::I64(try!(read_numeric_data::<R, i32>(rd)) as i64)),
        Marker::I64 => Value::Integer(Integer::I64(try!(read_numeric_data(rd)))),
        Marker::F32 => Value::Float(Float::F32(try!(read_numeric_data(rd)))),
        Marker::F64 => Value::Float(Float::F64(try!(read_numeric_data(rd)))),
        Marker::FixStr(len) => {
            let len = len as u32;
            let res = try!(read_str(rd, len));
            Value::String(res)
        }
        Marker::Str8 => {
            let len = try!(read_numeric_data::<R, u8>(rd)) as u32;
            let res = try!(read_str(rd, len));
            Value::String(res)
        }
        Marker::Str16 => {
            let len = try!(read_numeric_data::<R, u16>(rd)) as u32;
            let res = try!(read_str(rd, len));
            Value::String(res)
        }
        Marker::Str32 => {
            let len = try!(read_numeric_data(rd));
            let res = try!(read_str(rd, len));
            Value::String(res)
        }
        Marker::FixArray(len) => {
            let len = len as usize;
            let vec = try!(read_array(rd, len));
            Value::Array(vec)
        }
        Marker::Array16 => {
            let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
            let vec = try!(read_array(rd, len));
            Value::Array(vec)
        }
        Marker::Array32 => {
            let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
            let vec = try!(read_array(rd, len));
            Value::Array(vec)
        }
        Marker::FixMap(len) => {
            let len = len as usize;
            let map = try!(read_map(rd, len));
            Value::Map(map)
        }
        Marker::Map16 => {
            let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
            let map = try!(read_map(rd, len));
            Value::Map(map)
        }
        Marker::Map32 => {
            let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
            let map = try!(read_map(rd, len));
            Value::Map(map)
        }
        Marker::Bin8 => {
            let len = try!(read_numeric_data::<R, u8>(rd)) as usize;
            let vec = try!(read_bin_data(rd, len));
            Value::Binary(vec)
        }
        Marker::Bin16 => {
            let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
            let vec = try!(read_bin_data(rd, len));
            Value::Binary(vec)
        }
        Marker::Bin32 => {
            let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
            let vec = try!(read_bin_data(rd, len));
            Value::Binary(vec)
        }
        Marker::FixExt1 => {
            let len = 1 as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::FixExt2 => {
            let len = 2 as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::FixExt4 => {
            let len = 4 as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::FixExt8 => {
            let len = 8 as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::FixExt16 => {
            let len = 16 as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::Ext8 => {
            let len = try!(read_numeric_data::<R, u8>(rd)) as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::Ext16 => {
            let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::Ext32 => {
            let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
            let (ty, vec) = try!(read_ext_body(rd, len));
            Value::Ext(ty, vec)
        }
        Marker::Reserved => return Err(Error::TypeMismatch(Marker::Reserved)),
    };

    Ok(val)
}

} // mod value

pub use self::value::read_value;
