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

} // mod value

pub use self::value::read_value;
