use std::io::Read;

use Marker;
use super::{read_marker, read_data_u8, read_data_u16, read_data_u32, read_data_u64, ValueReadError};

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
fn stub() {}
