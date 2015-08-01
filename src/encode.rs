use std::convert::From;
use std::error::Error;
use std::fmt;
use std::io;
use std::io::Write;
use std::result::Result;

use byteorder;
use byteorder::WriteBytesExt;

use super::Marker;

#[path = "encode/value_ref.rs"]
pub mod value_ref;

/// Represents an error that can occur when attempting to write MessagePack'ed value into the write.
#[derive(Debug)]
pub struct WriteError(io::Error);

impl Error for WriteError {
    fn description(&self) -> &str { "error while writing MessagePack'ed value" }

    fn cause(&self) -> Option<&Error> {
        Some(&self.0)
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<byteorder::Error> for WriteError {
    fn from(err: byteorder::Error) -> WriteError {
        match err {
            byteorder::Error::UnexpectedEOF => {
                WriteError(io::Error::new(io::ErrorKind::Other, "unexpected EOF"))
            },
            byteorder::Error::Io(err) => WriteError(err),
        }
    }
}

/// Represents an error that can occur when attempting to write marker into the write.
#[derive(Debug)]
pub struct MarkerWriteError(WriteError);

impl Error for MarkerWriteError {
    fn description(&self) -> &str { "error while writing marker" }

    fn cause(&self) -> Option<&Error> {
        Some(&self.0)
    }
}

impl fmt::Display for MarkerWriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<byteorder::Error> for MarkerWriteError {
    fn from(err: byteorder::Error) -> MarkerWriteError {
        MarkerWriteError(From::from(err))
    }
}

/// Represents an error that can occur when attempting to write MessagePack'ed single-byte value.
#[derive(Debug)]
pub struct FixedValueWriteError(WriteError);

impl Error for FixedValueWriteError {
    fn description(&self) -> &str { "error while writing MessagePack'ed single-byte value" }

    fn cause(&self) -> Option<&Error> {
        Some(&self.0)
    }
}

impl fmt::Display for FixedValueWriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

/// Represents an error that can occur when attempring to write MessagePack'ed complex value into
/// the write.
#[derive(Debug)]
pub enum ValueWriteError {
    /// IO error while writing marker.
    InvalidMarkerWrite(WriteError),
    /// IO error while writing data.
    InvalidDataWrite(WriteError),
}

impl Error for ValueWriteError {
    fn description(&self) -> &str { "error while writing MessagePack'ed complex value" }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ValueWriteError::InvalidMarkerWrite(ref err) => Some(err),
            ValueWriteError::InvalidDataWrite(ref err) => Some(err),
        }
    }
}

impl fmt::Display for ValueWriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<MarkerWriteError> for ValueWriteError {
    fn from(err: MarkerWriteError) -> ValueWriteError {
        match err {
            MarkerWriteError(err) => ValueWriteError::InvalidMarkerWrite(err),
        }
    }
}

impl From<FixedValueWriteError> for ValueWriteError {
    fn from(err: FixedValueWriteError) -> ValueWriteError {
        match err {
            FixedValueWriteError(err) => ValueWriteError::InvalidMarkerWrite(err),
        }
    }
}

/// Attempts to write the given marker into the write and transforms any IO error to the special
/// kind of error.
fn write_marker<W>(wr: &mut W, marker: Marker) -> Result<(), MarkerWriteError>
    where W: Write
{
    wr.write_u8(marker.to_u8()).map_err(From::from)
}

/// Attempts to write the given fixed value (represented as marker) into the write and transforms
/// any IO error to the special kind of error.
fn write_fixval<W>(wr: &mut W, marker: Marker) -> Result<(), FixedValueWriteError>
    where W: Write
{
    wr.write_u8(marker.to_u8()).map_err(|err| FixedValueWriteError(From::from(err)))
}

/// Encodes and attempts to write a nil value into the given write.
///
/// According to the MessagePack specification, a nil value is represented as a single `0xc0` byte.
///
/// # Errors
///
/// This function will return `FixedValueWriteError` on any I/O error occurred while writing the nil
/// marker.
pub fn write_nil<W>(wr: &mut W) -> Result<(), FixedValueWriteError>
    where W: Write
{
    write_fixval(wr, Marker::Null)
}

/// Encodes and attempts to write a bool value into the given write.
///
/// According to the MessagePack specification, an encoded boolean value is represented as a single
/// byte.
///
/// # Errors
///
/// This function will return `FixedValueWriteError` on any I/O error occurred while writing the
/// boolean marker.
pub fn write_bool<W>(wr: &mut W, val: bool) -> Result<(), FixedValueWriteError>
    where W: Write
{
    match val {
        true  => write_fixval(wr, Marker::True),
        false => write_fixval(wr, Marker::False)
    }
}

/// Encodes and attempts to write an unsigned small integer value as a positive fixint into the
/// given write.
///
/// According to the MessagePack specification, a positive fixed integer value is represented using
/// a single byte in `[0x00; 0x7f]` range inclusively, prepended with a special marker mask.
///
/// The function is **strict** with the input arguments - it is the user's responsibility to check
/// if the value fits in the described range, otherwise it will panic.
///
/// If you are not sure if the value fits in the given range use `write_uint` instead, which
/// automatically selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `FixedValueWriteError` on any I/O error occurred while writing the
/// positive integer marker.
///
/// # Panics
///
/// Panics if `val` is greater than 127.
pub fn write_pfix<W>(wr: &mut W, val: u8) -> Result<(), FixedValueWriteError>
    where W: Write
{
    assert!(val < 128);

    write_fixval(wr, Marker::PositiveFixnum(val))
}

/// Encodes and attempts to write a negative small integer value as a negative fixnum into the
/// given write.
///
/// According to the MessagePack specification, a negative fixed integer value is represented using
/// a single byte in `[0xe0; 0xff]` range inclusively, prepended with a special marker mask.
///
/// The function is **strict** with the input arguments - it is the user's responsibility to check
/// if the value fits in the described range, otherwise it will panic.
///
/// If you are not sure if the value fits in the given range use `write_sint` instead, which
/// automatically selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `FixedValueWriteError` on any I/O error occurred while writing the
/// positive integer marker.
///
/// # Panics
///
/// Panics if `val` does not fit in `[-32; 0)` range.
pub fn write_nfix<W>(wr: &mut W, val: i8) -> Result<(), FixedValueWriteError>
    where W: Write
{
    assert!(-32 <= val && val < 0);

    write_fixval(wr, Marker::NegativeFixnum(val))
}

// TODO: Eliminate this or not?
macro_rules! make_write_data_fn {
    (deduce, $writer:ident, $encoder:ident, 0, $val:ident)
        => ($writer.$encoder($val););
    (deduce, $writer:ident, $encoder:ident, 1, $val:ident)
        => ($writer.$encoder::<byteorder::BigEndian>($val););
    (gen, $t:ty, $d:tt, $name:ident, $encoder:ident) => {
        fn $name<W>(wr: &mut W, val: $t) -> Result<(), ValueWriteError>
            where W: Write
        {
            match make_write_data_fn!(deduce, wr, $encoder, $d, val) {
                Ok(data) => Ok(data),
                Err(err) => Err(ValueWriteError::InvalidDataWrite(From::from(err))),
            }
        }
    };
    (u8,    $name:ident, $encoder:ident) => (make_write_data_fn!(gen, u8, 0, $name, $encoder););
    (i8,    $name:ident, $encoder:ident) => (make_write_data_fn!(gen, i8, 0, $name, $encoder););
    ($t:ty, $name:ident, $encoder:ident) => (make_write_data_fn!(gen, $t, 1, $name, $encoder););
}

make_write_data_fn!(u8,  write_data_u8,  write_u8);
make_write_data_fn!(u16, write_data_u16, write_u16);
make_write_data_fn!(u32, write_data_u32, write_u32);
make_write_data_fn!(u64, write_data_u64, write_u64);
make_write_data_fn!(i8,  write_data_i8,  write_i8);
make_write_data_fn!(i16, write_data_i16, write_i16);
make_write_data_fn!(i32, write_data_i32, write_i32);
make_write_data_fn!(i64, write_data_i64, write_i64);
make_write_data_fn!(f32, write_data_f32, write_f32);
make_write_data_fn!(f64, write_data_f64, write_f64);

/// Encodes and attempts to write an `u8` value as a 2-byte sequence into the given write.
///
/// The first byte becomes the marker and the second one will represent the data itself.
///
/// Note, that this function will encode the given value in 2-byte sequence no matter what, even if
/// the value can be represented using single byte as a positive fixnum.
///
/// If you need to fit the given buffer efficiently use `write_uint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
///
/// # Examples
/// ```
/// use rmp::encode::write_u8;
///
/// let mut buf = [0x00, 0x00];
///
/// write_u8(&mut &mut buf[..], 146).ok().unwrap();
/// assert_eq!([0xcc, 0x92], buf);
///
/// // Note, that 42 can be represented simply as `[0x2a]`, but the function emits 2-byte sequence.
/// write_u8(&mut &mut buf[..], 42).ok().unwrap();
/// assert_eq!([0xcc, 0x2a], buf);
/// ```
pub fn write_u8<W>(wr: &mut W, val: u8) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::U8));
    write_data_u8(wr, val)
}

/// Encodes and attempts to write an `u16` value strictly as a 3-byte sequence into the given write.
///
/// The first byte becomes the marker and the others will represent the data itself.
///
/// Note, that this function will encode the given value in 3-byte sequence no matter what, even if
/// the value can be represented using single byte as a positive fixnum.
///
/// If you need to fit the given buffer efficiently use `write_uint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_u16<W>(wr: &mut W, val: u16) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::U16));
    write_data_u16(wr, val)
}

/// Encodes and attempts to write an `u32` value strictly as a 5-byte sequence into the given write.
///
/// The first byte becomes the marker and the others will represent the data itself.
///
/// Note, that this function will encode the given value in 5-byte sequence no matter what, even if
/// the value can be represented using single byte as a positive fixnum.
///
/// If you need to fit the given buffer efficiently use `write_uint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_u32<W>(wr: &mut W, val: u32) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::U32));
    write_data_u32(wr, val)
}

/// Encodes and attempts to write an `u64` value strictly as a 9-byte sequence into the given write.
///
/// The first byte becomes the marker and the others will represent the data itself.
///
/// Note, that this function will encode the given value in 9-byte sequence no matter what, even if
/// the value can be represented using single byte as a positive fixnum.
///
/// If you need to fit the given buffer efficiently use `write_uint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_u64<W>(wr: &mut W, val: u64) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::U64));
    write_data_u64(wr, val)
}

/// Encodes and attempts to write an `i8` value as a 2-byte sequence into the given write.
///
/// The first byte becomes the marker and the second one will represent the data itself.
///
/// Note, that this function will encode the given value in 2-byte sequence no matter what, even if
/// the value can be represented using single byte as a fixnum. Also note, that the first byte will
/// always be the i8 marker (`0xd0`).
///
/// If you need to fit the given buffer efficiently use `write_sint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
///
/// # Examples
/// ```
/// use rmp::encode::write_i8;
///
/// let mut buf = [0x00, 0x00];
///
/// write_i8(&mut &mut buf[..], 42).ok().unwrap();
/// assert_eq!([0xd0, 0x2a], buf);
///
/// // Note, that -18 can be represented simply as `[0xee]`, but the function emits 2-byte sequence.
/// write_i8(&mut &mut buf[..], -18).ok().unwrap();
/// assert_eq!([0xd0, 0xee], buf);
/// ```
pub fn write_i8<W>(wr: &mut W, val: i8) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::I8));
    write_data_i8(wr, val)
}

/// Encodes and attempts to write an `i16` value as a 3-byte sequence into the given write.
///
/// The first byte becomes the marker and the others will represent the data itself.
///
/// Note, that this function will encode the given value in 3-byte sequence no matter what, even if
/// the value can be represented using single byte as a fixnum. Also note, that the first byte will
/// always be the i16 marker (`0xd1`).
///
/// If you need to fit the given buffer efficiently use `write_sint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_i16<W>(wr: &mut W, val: i16) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::I16));
    write_data_i16(wr, val)
}

/// Encodes and attempts to write an `i32` value as a 5-byte sequence into the given write.
///
/// The first byte becomes the marker and the others will represent the data itself.
///
/// Note, that this function will encode the given value in 5-byte sequence no matter what, even if
/// the value can be represented using single byte as a fixnum. Also note, that the first byte will
/// always be the i32 marker (`0xd2`).
///
/// If you need to fit the given buffer efficiently use `write_sint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_i32<W>(wr: &mut W, val: i32) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::I32));
    write_data_i32(wr, val)
}

/// Encodes and attempts to write an `i64` value as a 9-byte sequence into the given write.
///
/// The first byte becomes the marker and the others will represent the data itself.
///
/// Note, that this function will encode the given value in 9-byte sequence no matter what, even if
/// the value can be represented using single byte as a fixnum. Also note, that the first byte will
/// always be the i16 marker (`0xd3`).
///
/// If you need to fit the given buffer efficiently use `write_sint` instead, which automatically
/// selects the appropriate integer representation.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_i64<W>(wr: &mut W, val: i64) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::I64));
    write_data_i64(wr, val)
}

/// Encodes and attempts to write an `u64` value into the given write using the most efficient
/// representation, returning the marker used.
///
/// This function obeys the MessagePack specification, which requires that the serializer SHOULD use
/// the format which represents the data in the smallest number of bytes.
///
/// The first byte becomes the marker and the others (if present, up to 9) will represent the data
/// itself.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
// TODO: Replace with `match`?
pub fn write_uint<W>(wr: &mut W, val: u64) -> Result<Marker, ValueWriteError>
    where W: Write
{
    if val < 128 {
        let marker = Marker::PositiveFixnum(val as u8);

        try!(write_fixval(wr, marker));

        Ok(marker)
    } else if val < 256 {
        write_u8(wr, val as u8).and(Ok(Marker::U8))
    } else if val < 65536 {
        write_u16(wr, val as u16).and(Ok(Marker::U16))
    } else if val < 4294967296 {
        write_u32(wr, val as u32).and(Ok(Marker::U32))
    } else {
        write_u64(wr, val).and(Ok(Marker::U64))
    }
}

/// Encodes and attempts to write an `i64` value into the given write using the most efficient
/// representation, returning the marker used.
///
/// This function obeys the MessagePack specification, which requires that the serializer SHOULD use
/// the format which represents the data in the smallest number of bytes, with the exception of
/// sized/unsized types.
///
/// Note, that the function will **always** use signed integer representation even if the value can
/// be more efficiently represented using unsigned integer encoding.
///
/// The first byte becomes the marker and the others (if present, up to 9) will represent the data
/// itself.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_sint<W>(wr: &mut W, val: i64) -> Result<Marker, ValueWriteError>
    where W: Write
{
    if -32 <= val && val <= 0 {
        let marker = Marker::NegativeFixnum(val as i8);

        try!(write_fixval(wr, marker));

        Ok(marker)
    } else if -128 <= val && val < 128 {
        write_i8(wr, val as i8).and(Ok(Marker::I8))
    } else if -32768 <= val && val < 32768 {
        write_i16(wr, val as i16).and(Ok(Marker::I16))
    } else if -2147483648 <= val && val <= 2147483647 {
        write_i32(wr, val as i32).and(Ok(Marker::I32))
    } else {
        write_i64(wr, val).and(Ok(Marker::I64))
    }
}

#[allow(dead_code, unused_variables)]
fn write_int<W>(wr: &mut W, val: i64) -> Result<Marker, ValueWriteError>
    where W: Write
{
    unimplemented!()
}

/// Encodes and attempts to write an `f32` value as a 5-byte sequence into the given write.
///
/// The first byte becomes the `f32` marker and the others will represent the data itself.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_f32<W>(wr: &mut W, val: f32) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::F32));
    write_data_f32(wr, val)
}

/// Encodes and attempts to write an `f64` value as a 9-byte sequence into the given write.
///
/// The first byte becomes the `f64` marker and the others will represent the data itself.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_f64<W>(wr: &mut W, val: f64) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_marker(wr, Marker::F64));
    write_data_f64(wr, val)
}

/// Encodes and attempts to write the most efficient string length implementation to the given
/// write, returning the marker used.
///
/// This function is useful when you want to get full control for writing the data itself, for
/// example, when using non-blocking socket.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_str_len<W>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
    where W: Write
{
    if len < 32 {
        let marker = Marker::FixedString(len as u8);
        try!(write_fixval(wr, marker));
        Ok(marker)
    } else if len < 256 {
        try!(write_marker(wr, Marker::Str8));
        write_data_u8(wr, len as u8).and(Ok(Marker::Str8))
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Str16));
        write_data_u16(wr, len as u16).and(Ok(Marker::Str16))
    } else {
        try!(write_marker(wr, Marker::Str32));
        write_data_u32(wr, len).and(Ok(Marker::Str32))
    }
}

/// Encodes and attempts to write the most efficient string implementation to the given `Write`.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
// TODO: Docs, range check, example, visibility.
pub fn write_str<W>(wr: &mut W, data: &str) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_str_len(wr, data.len() as u32));
    wr.write_all(data.as_bytes()).map_err(|err| ValueWriteError::InvalidDataWrite(WriteError(err)))
}

/// Encodes and attempts to write the most efficient binary array length implementation to the given
/// write, returning the marker used.
///
/// This function is useful when you want to get full control for writing the data itself, for
/// example, when using non-blocking socket.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_bin_len<W>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
    where W: Write
{
    if len < 256 {
        try!(write_marker(wr, Marker::Bin8));
        write_data_u8(wr, len as u8).and(Ok(Marker::Bin8))
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Bin16));
        write_data_u16(wr, len as u16).and(Ok(Marker::Bin16))
    } else {
        try!(write_marker(wr, Marker::Bin32));
        write_data_u32(wr, len).and(Ok(Marker::Bin32))
    }
}

/// Encodes and attempts to write the most efficient binary implementation to the given `Write`.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
// TODO: Docs, range check, example, visibility.
pub fn write_bin<W>(wr: &mut W, data: &[u8]) -> Result<(), ValueWriteError>
    where W: Write
{
    try!(write_bin_len(wr, data.len() as u32));
    wr.write_all(data).map_err(|err| ValueWriteError::InvalidDataWrite(WriteError(err)))
}

/// Encodes and attempts to write the most efficient array length implementation to the given write,
/// returning the marker used.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_array_len<W>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
    where W: Write
{
    if len < 16 {
        let marker = Marker::FixedArray(len as u8);
        try!(write_fixval(wr, marker));
        Ok(marker)
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Array16));
        write_data_u16(wr, len as u16).and(Ok(Marker::Array16))
    } else {
        try!(write_marker(wr, Marker::Array32));
        write_data_u32(wr, len).and(Ok(Marker::Array32))
    }
}

/// Encodes and attempts to write the most efficient map length implementation to the given write,
/// returning the marker used.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
pub fn write_map_len<W>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
    where W: Write
{
    if len < 16 {
        let marker = Marker::FixedMap(len as u8);
        try!(write_fixval(wr, marker));
        Ok(marker)
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Map16));
        write_data_u16(wr, len as u16).and(Ok(Marker::Map16))
    } else {
        try!(write_marker(wr, Marker::Map32));
        write_data_u32(wr, len).and(Ok(Marker::Map32))
    }
}

/// Encodes and attempts to write the most efficient ext metadata implementation to the given
/// write, returning the marker used.
///
/// # Errors
///
/// This function will return `ValueWriteError` on any I/O error occurred while writing either the
/// marker or the data, except the EINTR, which is handled internally.
///
/// # Panics
///
/// Panics if `typeid` is negative, because it is reserved for future MessagePack extension
/// including 2-byte type information.
pub fn write_ext_meta<W>(wr: &mut W, len: u32, typeid: i8) -> Result<Marker, ValueWriteError>
    where W: Write
{
    assert!(typeid >= 0);

    let marker = match len {
        1 => {
            try!(write_marker(wr, Marker::FixExt1));
            Marker::FixExt1
        }
        2 => {
            try!(write_marker(wr, Marker::FixExt2));
            Marker::FixExt2
        }
        4 => {
            try!(write_marker(wr, Marker::FixExt4));
            Marker::FixExt4
        }
        8 => {
            try!(write_marker(wr, Marker::FixExt8));
            Marker::FixExt8
        }
        16 => {
            try!(write_marker(wr, Marker::FixExt16));
            Marker::FixExt16
        }
        len if len < 256 => {
            try!(write_marker(wr, Marker::Ext8));
            try!(write_data_u8(wr, len as u8));
            Marker::Ext8
        }
        len if len < 65536 => {
            try!(write_marker(wr, Marker::Ext16));
            try!(write_data_u16(wr, len as u16));
            Marker::Ext16
        }
        len => {
            try!(write_marker(wr, Marker::Ext32));
            try!(write_data_u32(wr, len));
            Marker::Ext32
        }
    };

    try!(write_data_i8(wr, typeid));

    Ok(marker)
}

pub mod value {

use std::convert::From;
use std::fmt;
use std::io::Write;
use std::result::Result;

pub use super::super::value::{
    Integer,
    Float,
    Value,
};

use super::*;

#[derive(Debug)]
pub enum Error {
    // TODO: Will be replaced with more concrete values.
    UnstableCommonError(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::UnstableCommonError(ref s) => s
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}


impl From<FixedValueWriteError> for Error {
    fn from(err: FixedValueWriteError) -> Error {
        match err {
            FixedValueWriteError(..) => Error::UnstableCommonError("fixed value error".to_string())
        }
    }
}

impl From<ValueWriteError> for Error {
    fn from(_: ValueWriteError) -> Error {
        Error::UnstableCommonError("value error".to_string())
    }
}

/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub fn write_value<W>(wr: &mut W, val: &Value) -> Result<(), Error>
    where W: Write
{
    match val {
        &Value::Nil => try!(write_nil(wr)),
        &Value::Boolean(val) => try!(write_bool(wr, val)),
        // TODO: Replace with generic write_int(...).
        &Value::Integer(Integer::U64(val)) => {
            try!(write_uint(wr, val));
        }
        &Value::Integer(Integer::I64(val)) => {
            try!(write_sint(wr, val));
        }
        // TODO: Replace with generic write_float(...).
        &Value::Float(Float::F32(val)) => try!(write_f32(wr, val)),
        &Value::Float(Float::F64(val)) => try!(write_f64(wr, val)),
        &Value::String(ref val) => {
            try!(write_str(wr, &val));
        }
        &Value::Binary(ref val) => {
            try!(write_bin(wr, &val));
        }
        &Value::Array(ref val) => {
            try!(write_array_len(wr, val.len() as u32));
            for item in val {
                try!(write_value(wr, item));
            }
        }
        &Value::Map(ref val) => {
            try!(write_map_len(wr, val.len() as u32));
            for &(ref key, ref val) in val {
                try!(write_value(wr, key));
                try!(write_value(wr, val));
            }
        }
        &Value::Ext(ty, ref data) => {
            try!(write_ext_meta(wr, data.len() as u32, ty));
            try!(wr.write_all(data).map_err(|err| ValueWriteError::InvalidDataWrite(WriteError(err))));
        }
    }

    Ok(())
}

} // mod value

pub mod serialize {

use serialize;

use std::fmt;
use std::io::Write;

use super::{
    write_nil,
    write_bool,
    write_uint,
    write_sint,
    write_f32,
    write_f64,
    write_str,
    write_array_len,
    write_map_len,
};

use super::{
    WriteError,
    FixedValueWriteError,
    ValueWriteError,
};

#[derive(Debug)]
pub enum Error {
    /// Failed to write MessagePack'ed single-byte value into the write.
    InvalidFixedValueWrite(WriteError),
    InvalidValueWrite(ValueWriteError),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "an error occurred while writing encoded value" }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::InvalidFixedValueWrite(ref err) => Some(err),
            Error::InvalidValueWrite(ref err) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}


impl From<FixedValueWriteError> for Error {
    fn from(err: FixedValueWriteError) -> Error {
        match err {
            FixedValueWriteError(err) => Error::InvalidFixedValueWrite(err)
        }
    }
}

impl From<ValueWriteError> for Error {
    fn from(err: ValueWriteError) -> Error {
        Error::InvalidValueWrite(err)
    }
}

/// Represents MessagePack serialization implementation.
///
/// # Note
///
/// MessagePack has no specification about how to encode variant types. Thus we are free to do
/// whatever we want, so the given chose may be not ideal for you.
///
/// Every Rust variant value can be represented as a tuple of index and a value.
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub struct Encoder<'a> {
    wr: &'a mut Write,
}

impl<'a> Encoder<'a> {
    /// Creates a new MessagePack encoder whose output will be written to the writer specified.
    pub fn new(wr: &'a mut Write) -> Encoder<'a> {
        Encoder {
            wr: wr,
        }
    }

    // TODO: An ability to borrow the `Write`.
}

impl<'a> serialize::Encoder for Encoder<'a> {
    type Error = Error;

    fn emit_nil(&mut self) -> Result<(), Error> {
        write_nil(&mut self.wr).map_err(From::from)
    }

    fn emit_bool(&mut self, val: bool) -> Result<(), Error> {
        write_bool(&mut self.wr, val).map_err(From::from)
    }

    fn emit_u8(&mut self, val: u8) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u16(&mut self, val: u16) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u32(&mut self, val: u32) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u64(&mut self, val: u64) -> Result<(), Error> {
        try!(write_uint(&mut self.wr, val));

        Ok(())
    }

    fn emit_usize(&mut self, val: usize) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_i8(&mut self, val: i8) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i16(&mut self, val: i16) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i32(&mut self, val: i32) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i64(&mut self, val: i64) -> Result<(), Error> {
        try!(write_sint(&mut self.wr, val));

        Ok(())
    }

    fn emit_isize(&mut self, val: isize) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_f32(&mut self, val: f32) -> Result<(), Error> {
        write_f32(&mut self.wr, val).map_err(From::from)
    }

    fn emit_f64(&mut self, val: f64) -> Result<(), Error> {
        write_f64(&mut self.wr, val).map_err(From::from)
    }

    // TODO: The implementation involves heap allocation and is unstable.
    fn emit_char(&mut self, val: char) -> Result<(), Error> {
        let mut buf = String::new();
        buf.push(val);
        self.emit_str(&buf)
    }

    fn emit_str(&mut self, val: &str) -> Result<(), Error> {
        write_str(&mut self.wr, val).map_err(From::from)
    }

    /// Encodes and attempts to write the enum value into the Write.
    ///
    /// Currently we encode variant types as a tuple of id with array of args, like: [id, [args...]]
    fn emit_enum<F>(&mut self, _name: &str, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        // Mark that we want to encode a variant type.
        try!(write_array_len(&mut self.wr, 2));

        // Delegate to the encoder of a concrete value.
        f(self)
    }

    /// Encodes and attempts to write a concrete variant value.
    fn emit_enum_variant<F>(&mut self, _name: &str, id: usize, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        // Encode a value position...
        try!(self.emit_usize(id));

        // ... and its arguments length.
        try!(write_array_len(&mut self.wr, len as u32));

        // Delegate to the encoder of a value args.
        f(self)
    }

    /// Encodes and attempts to write a concrete variant value arguments.
    fn emit_enum_variant_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_enum_struct_variant<F>(&mut self, _name: &str, _id: usize, _len: usize, _f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_enum_struct_variant_field<F>(&mut self, _name: &str, _idx: usize, _f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_struct<F>(&mut self, _name: &str, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        self.emit_tuple(len, f)
    }

    fn emit_struct_field<F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_array_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_tuple_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_tuple_struct<F>(&mut self, _name: &str, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        self.emit_tuple(len, f)
    }

    fn emit_tuple_struct_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_option<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_option_none(&mut self) -> Result<(), Error> {
        self.emit_nil()
    }

    fn emit_option_some<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    // TODO: Check len, overflow is possible.
    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_array_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_seq_elt<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_map_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_map_elt_key<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }
}

}
