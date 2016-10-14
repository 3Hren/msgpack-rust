use std::io::Read;

use Marker;
use super::{read_data_u8, read_data_u16, read_data_u32, read_data_u64, read_data_i8,
            read_data_i16, read_data_i32, read_data_i64, read_marker, ValueReadError};

/// Attempts to read a single byte from the given reader and to decode it as a positive fixnum
/// value.
///
/// According to the MessagePack specification, a positive fixed integer value is represented using
/// a single byte in `[0x00; 0x7f]` range inclusively, prepended with a special marker mask.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading the marker,
/// except the EINTR, which is handled internally.
///
/// It also returns `ValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_pfix<R: Read>(rd: &mut R) -> Result<u8, ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::FixPos(val) => Ok(val),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read exactly 2 bytes from the given reader and to decode them as `u8` value.
///
/// The first byte should be the marker and the second one should represent the data itself.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
///
/// It also returns `ValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
pub fn read_u8<R: Read>(rd: &mut R) -> Result<u8, ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::U8 => read_data_u8(rd),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read exactly 3 bytes from the given reader and to decode them as `u16` value.
///
/// The first byte should be the marker and the others should represent the data itself.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
///
/// It also returns `ValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_u16<R: Read>(rd: &mut R) -> Result<u16, ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::U16 => read_data_u16(rd),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read exactly 5 bytes from the given reader and to decode them as `u32` value.
///
/// The first byte should be the marker and the others should represent the data itself.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
///
/// It also returns `ValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_u32<R: Read>(rd: &mut R) -> Result<u32, ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::U32 => read_data_u32(rd),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read exactly 9 bytes from the given reader and to decode them as `u64` value.
///
/// The first byte should be the marker and the others should represent the data itself.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
///
/// It also returns `ValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_u64<R: Read>(rd: &mut R) -> Result<u64, ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::U64 => read_data_u64(rd),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read up to 9 bytes from the given reader and to decode them as `u64` value.
///
/// This function will try to read up to 9 bytes from the reader (1 for marker and up to 8 for data)
/// and interpret them as a big-endian u64.
///
/// Unlike the `read_u64`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
///
/// It also returns `ValueReadError::TypeMismatch` if the actual type is not an integer or it does
/// not fit in the given numeric range, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_uint<R: Read>(rd: &mut R) -> Result<u64, ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::FixPos(val) => Ok(val as u64),
        Marker::U8 => Ok(try!(read_data_u8(rd)) as u64),
        Marker::U16 => Ok(try!(read_data_u16(rd)) as u64),
        Marker::U32 => Ok(try!(read_data_u32(rd)) as u64),
        Marker::U64 => Ok(try!(read_data_u64(rd))),
        Marker::FixNeg(val) if val >= 0 => Ok(val as u64),
        Marker::I8 => {
            match try!(read_data_i8(rd)) {
                val if val >= 0 => Ok(val as u64),
                _ => Err(ValueReadError::TypeMismatch(Marker::I8)),
            }
        }
        Marker::I16 => {
            match try!(read_data_i16(rd)) {
                val if val >= 0 => Ok(val as u64),
                _ => Err(ValueReadError::TypeMismatch(Marker::I16)),
            }
        }
        Marker::I32 => {
            match try!(read_data_i32(rd)) {
                val if val >= 0 => Ok(val as u64),
                _ => Err(ValueReadError::TypeMismatch(Marker::I32)),
            }
        }
        Marker::I64 => {
            match try!(read_data_i64(rd)) {
                val if val >= 0 => Ok(val as u64),
                _ => Err(ValueReadError::TypeMismatch(Marker::I64)),
            }
        }
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

// read int that fit in T, type mismatch otherwise.
pub fn read_uint<T: TryInto<T, Err=TryFromIntError>, R: Read>(rd) -> Result<T, ValueReadError> {
    read_data(rd, try!(read_marker))
}

pub trait Integer {}

impl Integer for i8 {}

#[derive(Debug)]
pub struct TryFromIntError;

fn read_data<R: Read, T: TryFrom<u8> + TryFrom<>(rd: &mut R, marker: Marker) -> Result<T, Marker> {
    match marker {
        Marker::I8 => try!(read_data_i8(rd)).try_into().map_err(|_| Marker::I8)),
        marker => Err(marker),
    }
}

pub trait TryInto<T> {
    type Err;

    fn try_into(self) -> Result<T, Self::Err>;
}
