use std::io::{self, ErrorKind, Read};
use std::string::FromUtf8Error;

use rmp::Marker;
use rmp::decode::{read_marker, read_data_u8, read_data_u16, read_data_u32, read_data_u64,
                  read_data_i8, read_data_i16, read_data_i32, read_data_i64, read_data_f32,
                  read_data_f64, MarkerReadError, ValueReadError};

use Value;

/// This type represents all possible errors that can occur when deserializing a value.
#[derive(Debug)]
pub enum Error {
    /// Error while reading marker byte.
    InvalidMarkerRead(io::Error),
    /// Error while reading data.
    InvalidDataRead(io::Error),
    /// Decoded value type isn't equal with the expected one.
    TypeMismatch(Marker),
    /// Failed to properly decode UTF8.
    FromUtf8Error(FromUtf8Error),
}

impl Error {
    pub fn insufficient_bytes(&self) -> bool {
        match *self {
            Error::InvalidMarkerRead(ref err) if err.kind() == ErrorKind::UnexpectedEof => true,
            Error::InvalidDataRead(ref err) if err.kind() == ErrorKind::UnexpectedEof => true,
            Error::InvalidMarkerRead(..) |
            Error::InvalidDataRead(..) |
            Error::TypeMismatch(..) |
            Error::FromUtf8Error(..) => false,
        }
    }
}

// TODO: Soon.
// impl error::Error for Error {
//     fn description(&self) -> &str {
//         unimplemented!();
//     }
//
//     fn cause(&self) -> Option<&error::Error> {
//         unimplemented!();
//     }
// }

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
            ValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::FromUtf8Error(err)
    }
}

fn read_array_data<R: Read>(rd: &mut R, mut len: usize) -> Result<Vec<Value>, Error> {
    let mut vec = Vec::with_capacity(len);

    while len > 0 {
        vec.push(read_value(rd)?);
        len -= 1;
    }

    Ok(vec)
}

fn read_map_data<R: Read>(rd: &mut R, mut len: usize) -> Result<Vec<(Value, Value)>, Error> {
    let mut vec = Vec::with_capacity(len);

    while len > 0 {
        vec.push((read_value(rd)?, read_value(rd)?));
        len -= 1;
    }

    Ok(vec)
}

fn read_str_data<R: Read>(rd: &mut R, len: usize) -> Result<String, Error> {
    String::from_utf8(read_bin_data(rd, len)?).map_err(From::from)
}

fn read_bin_data<R: Read>(rd: &mut R, len: usize) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::with_capacity(len);
    buf.resize(len as usize, 0u8);
    rd.read_exact(&mut buf[..]).map_err(Error::InvalidDataRead)?;

    Ok(buf)
}

fn read_ext_body<R: Read>(rd: &mut R, len: usize) -> Result<(i8, Vec<u8>), Error> {
    let ty = read_data_i8(rd)?;
    let vec = read_bin_data(rd, len)?;

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
    let val = match read_marker(rd)? {
        Marker::Null => Value::Nil,
        Marker::True => Value::Boolean(true),
        Marker::False => Value::Boolean(false),
        Marker::FixPos(val) => Value::U64(val as u64),
        Marker::FixNeg(val) => Value::I64(val as i64),
        Marker::U8 => Value::U64(read_data_u8(rd)? as u64),
        Marker::U16 => Value::U64(read_data_u16(rd)? as u64),
        Marker::U32 => Value::U64(read_data_u32(rd)? as u64),
        Marker::U64 => Value::U64(read_data_u64(rd)?),
        Marker::I8 => Value::I64(read_data_i8(rd)? as i64),
        Marker::I16 => Value::I64(read_data_i16(rd)? as i64),
        Marker::I32 => Value::I64(read_data_i32(rd)? as i64),
        Marker::I64 => Value::I64(read_data_i64(rd)?),
        Marker::F32 => Value::F32(read_data_f32(rd)?),
        Marker::F64 => Value::F64(read_data_f64(rd)?),
        Marker::FixStr(len) => {
            let res = read_str_data(rd, len as usize)?;
            Value::String(res)
        }
        Marker::Str8 => {
            let len = read_data_u8(rd)?;
            let res = read_str_data(rd, len as usize)?;
            Value::String(res)
        }
        Marker::Str16 => {
            let len = read_data_u16(rd)?;
            let res = read_str_data(rd, len as usize)?;
            Value::String(res)
        }
        Marker::Str32 => {
            let len = read_data_u32(rd)?;
            let res = read_str_data(rd, len as usize)?;
            Value::String(res)
        }
        Marker::FixArray(len) => {
            let vec = read_array_data(rd, len as usize)?;
            Value::Array(vec)
        }
        Marker::Array16 => {
            let len = read_data_u16(rd)?;
            let vec = read_array_data(rd, len as usize)?;
            Value::Array(vec)
        }
        Marker::Array32 => {
            let len = read_data_u32(rd)?;
            let vec = read_array_data(rd, len as usize)?;
            Value::Array(vec)
        }
        Marker::FixMap(len) => {
            let map = read_map_data(rd, len as usize)?;
            Value::Map(map)
        }
        Marker::Map16 => {
            let len = read_data_u16(rd)?;
            let map = read_map_data(rd, len as usize)?;
            Value::Map(map)
        }
        Marker::Map32 => {
            let len = read_data_u32(rd)?;
            let map = read_map_data(rd, len as usize)?;
            Value::Map(map)
        }
        Marker::Bin8 => {
            let len = read_data_u8(rd)?;
            let vec = read_bin_data(rd, len as usize)?;
            Value::Binary(vec)
        }
        Marker::Bin16 => {
            let len = read_data_u16(rd)?;
            let vec = read_bin_data(rd, len as usize)?;
            Value::Binary(vec)
        }
        Marker::Bin32 => {
            let len = read_data_u32(rd)?;
            let vec = read_bin_data(rd, len as usize)?;
            Value::Binary(vec)
        }
        Marker::FixExt1 => {
            let len = 1 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt2 => {
            let len = 2 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt4 => {
            let len = 4 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt8 => {
            let len = 8 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::FixExt16 => {
            let len = 16 as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Ext8 => {
            let len = read_data_u8(rd)? as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Ext16 => {
            let len = read_data_u16(rd)? as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Ext32 => {
            let len = read_data_u32(rd)? as usize;
            let (ty, vec) = read_ext_body(rd, len)?;
            Value::Ext(ty, vec)
        }
        Marker::Reserved => return Err(Error::TypeMismatch(Marker::Reserved)),
    };

    Ok(val)
}
