//! # Note
//!
//! Most of the function defined in this module will silently handle interruption error (EINTR)
//! received from the given `Read` to be in consistent state with the `Write::write_all` method in
//! the standard library.
//!
//! Any other error would immediately interrupt the parsing process. If your reader can results in
//! I/O error and simultaneously be a recoverable state (for example, when reading from
//! non-blocking socket and it returns EWOULDBLOCK) be sure that you buffer the data externally
//! to avoid data loss (using `BufRead` readers with manual consuming or some other way).
//
// TODO: So what, we can either force the user to manage data state externally or to restrict reader
// type to BufRead for primitive cases (read_[nil, bool, u*, i*, f*]) where required size is known.

use std::error::Error;
use std::fmt;
use std::io;
use std::io::Read;
use std::result::Result;
use std::str::{Utf8Error, from_utf8};

use byteorder;
use byteorder::ReadBytesExt;

use super::Marker;

#[path = "decode/value_ref.rs"]
pub mod value_ref;
pub use self::value_ref::read_value_ref;

/// Represents an error that can occur when attempting to read bytes from the reader.
///
/// This is a thin wrapper over the standard `io::Error` type. Namely, it adds one additional error
/// case: an unexpected EOF.
#[derive(Debug)]
pub enum ReadError {
    /// Unexpected end of file reached while reading bytes.
    UnexpectedEOF,
    /// I/O error occurred while reading bytes.
    Io(io::Error),
}

impl Error for ReadError {
    fn description(&self) -> &str {
        match *self {
            ReadError::UnexpectedEOF => "unexpected end of file while reading MessagePack value",
            // TODO: Probably we should give here a short description that I/O error occurs.
            ReadError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ReadError::UnexpectedEOF => None,
            ReadError::Io(ref err) => Some(err),
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}

impl From<byteorder::Error> for ReadError {
    fn from(err: byteorder::Error) -> ReadError {
        match err {
            byteorder::Error::UnexpectedEOF => ReadError::UnexpectedEOF,
            byteorder::Error::Io(err) => ReadError::Io(err),
        }
    }
}

/// Represents an error that can occur when attempting to read a MessagePack marker from the reader.
///
/// This is a thin wrapper over the standard `io::Error` type. Namely, it adds one additional error
/// case: an unexpected EOF.
#[derive(Debug)]
pub enum MarkerReadError {
    /// Unexpected end of file reached while reading the marker.
    UnexpectedEOF,
    /// I/O error occurred while reading the marker.
    Io(io::Error),
}

impl Error for MarkerReadError {
    fn description(&self) -> &str {
        match *self {
            MarkerReadError::UnexpectedEOF => "unexpected end of file while reading MessagePack marker",
            MarkerReadError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            MarkerReadError::UnexpectedEOF => None,
            MarkerReadError::Io(ref err) => Some(err),
        }
    }
}

impl fmt::Display for MarkerReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}


impl From<byteorder::Error> for MarkerReadError {
    fn from(err: byteorder::Error) -> MarkerReadError {
        match err {
            byteorder::Error::UnexpectedEOF => MarkerReadError::UnexpectedEOF,
            byteorder::Error::Io(err) => MarkerReadError::Io(err),
        }
    }
}

impl From<MarkerReadError> for ReadError {
    fn from(err: MarkerReadError) -> ReadError {
        match err {
            MarkerReadError::UnexpectedEOF => ReadError::UnexpectedEOF,
            MarkerReadError::Io(err) => ReadError::Io(err),
        }
    }
}

/// Represents an error that can occur when attempting to read a MessagePack'ed single-byte value
/// from the reader.
#[derive(Debug)]
pub enum FixedValueReadError {
    /// Unexpected end of file reached while reading the value.
    UnexpectedEOF,
    /// I/O error occurred while reading the value.
    Io(io::Error),
    /// The type decoded isn't match with the expected one.
    TypeMismatch(Marker),
}

impl Error for FixedValueReadError {
    fn description(&self) -> &str {
        use self::FixedValueReadError::*;
        match *self {
            UnexpectedEOF => "unexpected end of file while reading MessagePack single-byte value",
            Io(ref err) => err.description(),
            TypeMismatch(_) => "the type decoded isn't match with the expected one",
        }
    }

    fn cause(&self) -> Option<&Error> {
        use self::FixedValueReadError::*;
        match *self {
            UnexpectedEOF => None,
            Io(ref err) => Some(err),
            TypeMismatch(_) => None,
        }
    }
}

impl fmt::Display for FixedValueReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<MarkerReadError> for FixedValueReadError {
    fn from(err: MarkerReadError) -> FixedValueReadError {
        match err {
            MarkerReadError::UnexpectedEOF => FixedValueReadError::UnexpectedEOF,
            MarkerReadError::Io(err) => FixedValueReadError::Io(err),
        }
    }
}

/// Represents an error that can occur when attempting to read a MessagePack'ed complex value from
/// the reader.
#[derive(Debug)]
pub enum ValueReadError {
    /// Failed to read the marker.
    InvalidMarkerRead(ReadError),
    /// Failed to read the data.
    InvalidDataRead(ReadError),
    /// The type decoded isn't match with the expected one.
    TypeMismatch(Marker),
}

impl Error for ValueReadError {
    fn description(&self) -> &str {
        match *self {
            ValueReadError::InvalidMarkerRead(..) => "failed to read MessagePack marker",
            ValueReadError::InvalidDataRead(..) => "failed to read MessagePack data",
            ValueReadError::TypeMismatch(..) => "the type decoded isn't match with the expected one",
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            ValueReadError::InvalidMarkerRead(ref err) => Some(err),
            ValueReadError::InvalidDataRead(ref err) => Some(err),
            ValueReadError::TypeMismatch(..) => None,
        }
    }
}

impl fmt::Display for ValueReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl From<MarkerReadError> for ValueReadError {
    fn from(err: MarkerReadError) -> ValueReadError {
        ValueReadError::InvalidMarkerRead(From::from(err))
    }
}

#[derive(Debug)]
pub enum DecodeStringError<'a> {
    InvalidMarkerRead(ReadError),
    InvalidDataRead(ReadError),
    TypeMismatch(Marker),
    /// The given buffer is not large enough to accumulate the specified amount of bytes.
    BufferSizeTooSmall(u32),
    InvalidDataCopy(&'a [u8], ReadError),
    InvalidUtf8(&'a [u8], Utf8Error),
}

impl<'a> Error for DecodeStringError<'a> {
    fn description(&self) -> &str { "error while decoding string" }

    fn cause(&self) -> Option<&Error> {
        use self::DecodeStringError::*;
        match *self {
            InvalidMarkerRead(ref err) => Some(err),
            InvalidDataRead(ref err) => Some(err),
            TypeMismatch(_) => None,
            BufferSizeTooSmall(_) => None,
            InvalidDataCopy(_, ref err) => Some(err),
            InvalidUtf8(_, ref err) => Some(err),
        }
    }
}

impl<'a> fmt::Display for DecodeStringError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl<'a> From<ValueReadError> for DecodeStringError<'a> {
    fn from(err: ValueReadError) -> DecodeStringError<'a> {
        match err {
            ValueReadError::InvalidMarkerRead(err) => DecodeStringError::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => DecodeStringError::InvalidDataRead(err),
            ValueReadError::TypeMismatch(marker) => DecodeStringError::TypeMismatch(marker),
        }
    }
}

/// Attempts to read a single byte from the given reader and decodes it as a MessagePack marker.
fn read_marker<R>(rd: &mut R) -> Result<Marker, MarkerReadError>
    where R: Read
{
    match rd.read_u8() {
        Ok(val) => Ok(Marker::from_u8(val)),
        Err(err) => Err(From::from(err)),
    }
}

/// Attempts to read a single byte from the given reader and to decode it as a nil value.
///
/// According to the MessagePack specification, a nil value is represented as a single `0xc0` byte.
///
/// # Errors
///
/// This function will return `FixedValueReadError` on any I/O error while reading the nil marker,
/// except the EINTR, which is handled internally.
///
/// It also returns `FixedValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_nil<R>(rd: &mut R) -> Result<(), FixedValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::Null => Ok(()),
        marker => Err(FixedValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read a single byte from the given reader and to decode it as a boolean value.
///
/// According to the MessagePack specification, an encoded boolean value is represented as a single
/// byte.
///
/// # Errors
///
/// This function will return `FixedValueReadError` on any I/O error while reading the bool marker,
/// except the EINTR, which is handled internally.
///
/// It also returns `FixedValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_bool<R>(rd: &mut R) -> Result<bool, FixedValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::True => Ok(true),
        Marker::False => Ok(false),
        marker => Err(FixedValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read a single byte from the given reader and to decode it as a positive fixnum
/// value.
///
/// According to the MessagePack specification, a positive fixed integer value is represented using
/// a single byte in `[0x00; 0x7f]` range inclusively, prepended with a special marker mask.
///
/// # Errors
///
/// This function will return `FixedValueReadError` on any I/O error while reading the marker,
/// except the EINTR, which is handled internally.
///
/// It also returns `FixedValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_pfix<R>(rd: &mut R) -> Result<u8, FixedValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixPos(val) => Ok(val),
        marker => Err(FixedValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read a single byte from the given reader and to decode it as a negative fixnum
/// value.
///
/// According to the MessagePack specification, a negative fixed integer value is represented using
/// a single byte in `[0xe0; 0xff]` range inclusively, prepended with a special marker mask.
///
/// # Errors
///
/// This function will return `FixedValueReadError` on any I/O error while reading the marker,
/// except the EINTR, which is handled internally.
///
/// It also returns `FixedValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_nfix<R>(rd: &mut R) -> Result<i8, FixedValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixNeg(val) => Ok(val),
        marker => Err(FixedValueReadError::TypeMismatch(marker)),
    }
}

trait BigEndianRead {
    fn read<R: Read>(rd: &mut R) -> Result<Self, byteorder::Error>;
}

impl BigEndianRead for u8 {
    fn read<R: Read>(rd: &mut R) -> Result<u8, byteorder::Error> {
        rd.read_u8()
    }
}

impl BigEndianRead for u16 {
    fn read<R: Read>(rd: &mut R) -> Result<u16, byteorder::Error> {
        rd.read_u16::<byteorder::BigEndian>()
    }
}

impl BigEndianRead for u32 {
    fn read<R: Read>(rd: &mut R) -> Result<u32, byteorder::Error> {
        rd.read_u32::<byteorder::BigEndian>()
    }
}

impl BigEndianRead for u64 {
    fn read<R: Read>(rd: &mut R) -> Result<u64, byteorder::Error> {
        rd.read_u64::<byteorder::BigEndian>()
    }
}

impl BigEndianRead for i8 {
    fn read<R: Read>(rd: &mut R) -> Result<i8, byteorder::Error> {
        rd.read_i8()
    }
}

impl BigEndianRead for i16 {
    fn read<R: Read>(rd: &mut R) -> Result<i16, byteorder::Error> {
        rd.read_i16::<byteorder::BigEndian>()
    }
}

impl BigEndianRead for i32 {
    fn read<R: Read>(rd: &mut R) -> Result<i32, byteorder::Error> {
        rd.read_i32::<byteorder::BigEndian>()
    }
}

impl BigEndianRead for i64 {
    fn read<R: Read>(rd: &mut R) -> Result<i64, byteorder::Error> {
        rd.read_i64::<byteorder::BigEndian>()
    }
}

impl BigEndianRead for f32 {
    fn read<R: Read>(rd: &mut R) -> Result<f32, byteorder::Error> {
        rd.read_f32::<byteorder::BigEndian>()
    }
}

impl BigEndianRead for f64 {
    fn read<R: Read>(rd: &mut R) -> Result<f64, byteorder::Error> {
        rd.read_f64::<byteorder::BigEndian>()
    }
}

fn read_numeric_data<R, D>(rd: &mut R) -> Result<D, ValueReadError>
    where R: Read,
          D: BigEndianRead
{
    match D::read(rd) {
        Ok(data) => Ok(data),
        Err(err) => Err(ValueReadError::InvalidDataRead(From::from(err))),
    }
}

#[inline]
fn read_numeric<R, D>(rd: &mut R, marker: Marker) -> Result<D, ValueReadError>
    where R: Read,
          D: BigEndianRead
{
    match try!(read_marker(rd)) {
        actual if actual == marker => read_numeric_data(rd).map_err(From::from),
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
pub fn read_u8<R>(rd: &mut R) -> Result<u8, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::U8)
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
pub fn read_u16<R>(rd: &mut R) -> Result<u16, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::U16)
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
pub fn read_u32<R>(rd: &mut R) -> Result<u32, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::U32)
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
pub fn read_u64<R>(rd: &mut R) -> Result<u64, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::U64)
}

/// Attempts to read exactly 2 bytes from the given reader and to decode them as `i8` value.
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
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_i8<R>(rd: &mut R) -> Result<i8, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::I8)
}

/// Attempts to read exactly 3 bytes from the given reader and to decode them as `i16` value.
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
pub fn read_i16<R>(rd: &mut R) -> Result<i16, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::I16)
}

/// Attempts to read exactly 5 bytes from the given reader and to decode them as `i32` value.
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
pub fn read_i32<R>(rd: &mut R) -> Result<i32, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::I32)
}

/// Attempts to read exactly 9 bytes from the given reader and to decode them as `i64` value.
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
pub fn read_i64<R>(rd: &mut R) -> Result<i64, ValueReadError>
    where R: Read
{
    read_numeric(rd, Marker::I64)
}

/// Attempts to read up to 2 bytes from the given reader and to decode them as `u8` value.
///
/// Unlike the `read_u8`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// Note, that trying to decode signed integers will result in `TypeMismatch` error even if the
/// value fits in `u8`.
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
pub fn read_u8_loosely<R>(rd: &mut R) -> Result<u8, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixPos(val) => Ok(val),
        Marker::U8 => Ok(try!(read_numeric_data(rd))),
        marker     => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read up to 3 bytes from the given reader and to decode them as `u16` value.
///
/// Unlike the `read_u16`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// Note, that trying to decode signed integers will result in `TypeMismatch` error even if the
/// value fits in `u16`.
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
pub fn read_u16_loosely<R>(rd: &mut R) -> Result<u16, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixPos(val) => Ok(val as u16),
        Marker::U8  => Ok(try!(read_numeric_data::<R, u8>(rd)) as u16),
        Marker::U16 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read up to 5 bytes from the given reader and to decode them as `u32` value.
///
/// Unlike the `read_u32`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// Note, that trying to decode signed integers will result in `TypeMismatch` error even if the
/// value fits in `u32`.
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
pub fn read_u32_loosely<R>(rd: &mut R) -> Result<u32, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixPos(val) => Ok(val as u32),
        Marker::U8  => Ok(try!(read_numeric_data::<R, u8>(rd))  as u32),
        Marker::U16 => Ok(try!(read_numeric_data::<R, u16>(rd)) as u32),
        Marker::U32 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker)),
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
/// Note, that trying to decode signed integers will result in `TypeMismatch` error even if the
/// value fits in `u64`.
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
pub fn read_u64_loosely<R>(rd: &mut R) -> Result<u64, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixPos(val) => Ok(val as u64),
        Marker::U8  => Ok(try!(read_numeric_data::<R, u8>(rd))  as u64),
        Marker::U16 => Ok(try!(read_numeric_data::<R, u16>(rd)) as u64),
        Marker::U32 => Ok(try!(read_numeric_data::<R, u32>(rd)) as u64),
        Marker::U64 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read up to 2 bytes from the given reader and to decode them as `i8` value.
///
/// Unlike the `read_i8`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// Note, that trying to decode unsigned integers will result in `TypeMismatch` error even if the
/// value fits in `i8`.
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
pub fn read_i8_loosely<R>(rd: &mut R) -> Result<i8, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixNeg(val) => Ok(val),
        Marker::I8  => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read up to 3 bytes from the given reader and to decode them as `i16` value.
///
/// Unlike the `read_i16`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// Note, that trying to decode unsigned integers will result in `TypeMismatch` error even if the
/// value fits in `i16`.
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
pub fn read_i16_loosely<R>(rd: &mut R) -> Result<i16, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixNeg(val) => Ok(val as i16),
        Marker::I8  => Ok(try!(read_numeric_data::<R, i8>(rd)) as i16),
        Marker::I16 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read up to 5 bytes from the given reader and to decode them as `i32` value.
///
/// Unlike the `read_i32`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// Note, that trying to decode unsigned integers will result in `TypeMismatch` error even if the
/// value fits in `i32`.
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
pub fn read_i32_loosely<R>(rd: &mut R) -> Result<i32, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixNeg(val) => Ok(val as i32),
        Marker::I8  => Ok(try!(read_numeric_data::<R, i8>(rd))  as i32),
        Marker::I16 => Ok(try!(read_numeric_data::<R, i16>(rd)) as i32),
        Marker::I32 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read up to 9 bytes from the given reader and to decode them as `i64` value.
///
/// This function will try to read up to 9 bytes from the reader (1 for marker and up to 8 for data)
/// and interpret them as a big-endian i64.
///
/// Unlike the `read_i64`, this function weakens type restrictions, allowing you to safely decode
/// packed values even if you aren't sure about the actual type.
///
/// Note, that trying to decode signed integers will result in `TypeMismatch` error even if the
/// value fits in `i64`.
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
pub fn read_i64_loosely<R>(rd: &mut R) -> Result<i64, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixNeg(val) => Ok(val as i64),
        Marker::I8  => Ok(try!(read_numeric_data::<R, i8>(rd))  as i64),
        Marker::I16 => Ok(try!(read_numeric_data::<R, i16>(rd)) as i64),
        Marker::I32 => Ok(try!(read_numeric_data::<R, i32>(rd)) as i64),
        Marker::I64 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker)),
    }
}

use super::value::Integer;

macro_rules! make_read_int_fit {
    ($name:ident, $ty:ty) => {
        #[allow(unused_comparisons)]
        fn $name<R>(rd: &mut R) -> Result<$ty, ValueReadError>
            where R: Read
        {
            let marker = try!(read_marker(rd));
            let val = match marker {
                // TODO: From trait.
                Marker::FixPos(val) => Ok(Integer::U64(val as u64)),
                Marker::FixNeg(val) => Ok(Integer::I64(val as i64)),
                Marker::U8  => Ok(Integer::U64(try!(read_numeric_data::<R, u8>(rd))  as u64)),
                Marker::I8  => Ok(Integer::I64(try!(read_numeric_data::<R, i8>(rd))  as i64)),
                Marker::U16 => Ok(Integer::U64(try!(read_numeric_data::<R, u16>(rd)) as u64)),
                Marker::I16 => Ok(Integer::I64(try!(read_numeric_data::<R, i16>(rd)) as i64)),
                Marker::U32 => Ok(Integer::U64(try!(read_numeric_data::<R, u32>(rd)) as u64)),
                Marker::I32 => Ok(Integer::I64(try!(read_numeric_data::<R, i32>(rd)) as i64)),
                Marker::U64 => Ok(Integer::U64(try!(read_numeric_data::<R, u64>(rd)) as u64)),
                Marker::I64 => Ok(Integer::I64(try!(read_numeric_data::<R, i64>(rd)) as i64)),
                marker => Err(ValueReadError::TypeMismatch(marker)),
            };

            match try!(val) {
                Integer::I64(v) => {
                    let res = v as $ty;

                    if v == res as i64 && (res > 0) == (v > 0) {
                        Ok(res)
                    } else {
                        Err(ValueReadError::TypeMismatch(marker))
                    }
                }
                Integer::U64(v) => {
                    let res = v as $ty;

                    if v == res as u64 && res >= 0 {
                        Ok(res)
                    } else {
                        Err(ValueReadError::TypeMismatch(marker))
                    }
                }
            }
        }
    }
}

make_read_int_fit!(read_i8_fit, i8);
make_read_int_fit!(read_i16_fit, i16);
make_read_int_fit!(read_i32_fit, i32);
make_read_int_fit!(read_i64_fit, i64);
make_read_int_fit!(read_u8_fit, u8);
make_read_int_fit!(read_u16_fit, u16);
make_read_int_fit!(read_u32_fit, u32);
make_read_int_fit!(read_u64_fit, u64);

/// Attempts to read exactly 5 bytes from the given reader and to decode them as `f32` value.
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
pub fn read_f32<R>(rd: &mut R) -> Result<f32, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::F32 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read exactly 9 bytes from the given reader and to decode them as `f64` value.
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
pub fn read_f64<R>(rd: &mut R) -> Result<f64, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::F64 => Ok(try!(read_numeric_data(rd))),
        marker      => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read up to 9 bytes from the given reader and to decode them as a string `u32` size
/// value.
///
/// According to the MessagePack specification, the string format family stores an byte array in 1,
/// 2, 3, or 5 bytes of extra bytes in addition to the size of the byte array.
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
pub fn read_str_len<R>(rd: &mut R) -> Result<u32, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixStr(size) => Ok(size as u32),
        Marker::Str8  => Ok(try!(read_numeric_data::<R, u8>(rd))  as u32),
        Marker::Str16 => Ok(try!(read_numeric_data::<R, u16>(rd)) as u32),
        Marker::Str32 => Ok(try!(read_numeric_data(rd))),
        marker        => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read a string data from the given reader and copy it to the buffer provided.
///
/// On success returns a borrowed string type, allowing to view the copyed bytes as properly utf-8
/// string.
/// According to the spec, the string's data must to be encoded using utf-8.
///
/// # Errors
///
/// Returns `Err` in the following cases:
///
///  - if any IO error (including unexpected EOF) occurs, while reading an `rd`, except the EINTR,
///    which is handled internally.
///  - if the `out` buffer size is not large enough to keep all the data copyed.
///  - if the data is not utf-8, with a description as to why the provided data is not utf-8 and
///    with a size of bytes actually copyed to be able to get them from `out`.
///
/// # Examples
/// ```
/// use rmp::decode::read_str;
///
/// let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
/// let mut out = [0u8; 16];
///
/// assert_eq!("le message", read_str(&mut &buf[..], &mut &mut out[..]).unwrap());
/// ```
///
/// # Unstable
///
/// This function is **unstable**, because it needs review.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
// TODO: Stabilize. Mark error values for each error case (in docs).
pub fn read_str<'r, R>(rd: &mut R, mut buf: &'r mut [u8]) -> Result<&'r str, DecodeStringError<'r>>
    where R: Read
{
    let len = try!(read_str_len(rd));
    let ulen = len as usize;

    if buf.len() < ulen {
        return Err(DecodeStringError::BufferSizeTooSmall(len))
    }

    read_str_data(rd, len, &mut buf[0..ulen])
}

fn read_str_data<'r, R>(rd: &mut R, len: u32, buf: &'r mut[u8]) -> Result<&'r str, DecodeStringError<'r>>
    where R: Read
{
    debug_assert_eq!(len as usize, buf.len());

    // Trying to copy exact `len` bytes.
    match read_full(rd, buf) {
        Ok(n) if n == buf.len() => {
            match from_utf8(buf) {
                Ok(decoded) => Ok(decoded),
                Err(err) => Err(DecodeStringError::InvalidUtf8(buf, err)),
            }
        }
        Ok(n) => Err(DecodeStringError::InvalidDataCopy(&buf[..n], ReadError::UnexpectedEOF)),
        Err(err) => Err(DecodeStringError::InvalidDataRead(ReadError::Io(From::from(err)))),
    }
}

/// Attempts to read and decode a string value from the reader, returning a borrowed slice from it.
///
// TODO: it is better to return &str; may panic on len mismatch; extend documentation.
// TODO: Also it's possible to implement all borrowing functions for all `BufRead` implementors.
// TODO: It's not necessary to use cursor, use slices instead.
pub fn read_str_ref(rd: &[u8]) -> Result<&[u8], DecodeStringError> {
    let mut cur = io::Cursor::new(rd);
    let len = try!(read_str_len(&mut cur));
    let start = cur.position() as usize;
    Ok(&rd[start .. start + len as usize])
}

/// Attempts to read up to 5 bytes from the given reader and to decode them as a big-endian u32
/// array size.
///
/// Array format family stores a sequence of elements in 1, 3, or 5 bytes of extra bytes in addition
/// to the elements.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
// TODO: Docs.
// NOTE: EINTR is managed internally.
pub fn read_array_size<R>(rd: &mut R) -> Result<u32, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixArray(size) => Ok(size as u32),
        Marker::Array16 => Ok(try!(read_numeric_data::<R, u16>(rd)) as u32),
        Marker::Array32 => Ok(try!(read_numeric_data(rd))),
        marker => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read up to 5 bytes from the given reader and to decode them as a big-endian u32
/// map size.
///
/// Map format family stores a sequence of elements in 1, 3, or 5 bytes of extra bytes in addition
/// to the elements.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
// TODO: Docs.
pub fn read_map_size<R>(rd: &mut R) -> Result<u32, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixMap(size) => Ok(size as u32),
        Marker::Map16 => Ok(try!(read_numeric_data::<R, u16>(rd)) as u32),
        Marker::Map32 => Ok(try!(read_numeric_data(rd))),
        marker => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
// TODO: Docs.
pub fn read_bin_len<R>(rd: &mut R) -> Result<u32, ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::Bin8  => Ok(try!(read_numeric_data::<R, u8>(rd)) as u32),
        Marker::Bin16 => Ok(try!(read_numeric_data::<R, u16>(rd)) as u32),
        Marker::Bin32 => Ok(try!(read_numeric_data(rd))),
        marker        => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
// TODO: Docs; not sure about naming.
pub fn read_bin_borrow(rd: &[u8]) -> Result<&[u8], ValueReadError> {
    let mut cur = io::Cursor::new(rd);
    let len = try!(read_bin_len(&mut cur)) as usize;

    let pos = cur.position() as usize;

    if rd.len() < pos + len {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF))
    } else {
        Ok(&rd[pos .. pos + len])
    }
}

/// Attempts to read exactly 3 bytes from the given reader and interpret them as a fixext1 type
/// with data attached.
///
/// According to the MessagePack specification, a fixext1 stores an integer and a byte array whose
/// length is 1 byte. Its marker byte is `0xd4`.
///
/// Note, that this function copies a byte array from the reader to the output `u8` variable.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_fixext1<R>(rd: &mut R) -> Result<(i8, u8), ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt1 => {
            let ty   = try!(read_numeric_data(rd));
            let data = try!(read_numeric_data(rd));
            Ok((ty, data))
        }
        marker => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read exactly 4 bytes from the given reader and interpret them as a fixext2 type
/// with data attached.
///
/// According to the MessagePack specification, a fixext2 stores an integer and a byte array whose
/// length is 2 bytes. Its marker byte is `0xd5`.
///
/// Note, that this function copies a byte array from the reader to the output buffer, which is
/// unlikely if you want zero-copy functionality.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data.
pub fn read_fixext2<R>(rd: &mut R) -> Result<(i8, [u8; 2]), ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt2 => {
            let mut buf = [0; 2];
            read_fixext_data(rd, &mut buf).map(|ty| (ty, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read exactly 6 bytes from the given reader and interpret them as a fixext4 type
/// with data attached.
///
/// According to the MessagePack specification, a fixext4 stores an integer and a byte array whose
/// length is 4 bytes. Its marker byte is `0xd6`.
///
/// Note, that this function copies a byte array from the reader to the output buffer, which is
/// unlikely if you want zero-copy functionality.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
pub fn read_fixext4<R>(rd: &mut R) -> Result<(i8, [u8; 4]), ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt4 => {
            let mut buf = [0; 4];
            read_fixext_data(rd, &mut buf).map(|ty| (ty, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read exactly 10 bytes from the given reader and interpret them as a fixext8 type
/// with data attached.
///
/// According to the MessagePack specification, a fixext8 stores an integer and a byte array whose
/// length is 8 bytes. Its marker byte is `0xd7`.
///
/// Note, that this function copies a byte array from the reader to the output buffer, which is
/// unlikely if you want zero-copy functionality.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
pub fn read_fixext8<R>(rd: &mut R) -> Result<(i8, [u8; 8]), ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt8 => {
            let mut buf = [0; 8];
            read_fixext_data(rd, &mut buf).map(|ty| (ty, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker))
    }
}

/// Attempts to read exactly 18 bytes from the given reader and interpret them as a fixext16 type
/// with data attached.
///
/// According to the MessagePack specification, a fixext16 stores an integer and a byte array whose
/// length is 16 bytes. Its marker byte is `0xd8`.
///
/// Note, that this function copies a byte array from the reader to the output buffer, which is
/// unlikely if you want zero-copy functionality.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
pub fn read_fixext16<R>(rd: &mut R) -> Result<(i8, [u8; 16]), ValueReadError>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt16 => {
            let mut buf = [0; 16];
            read_fixext_data(rd, &mut buf).map(|ty| (ty, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker))
    }
}

fn read_fixext_data<R>(rd: &mut R, buf: &mut [u8]) -> Result<i8, ValueReadError>
    where R: Read
{
    let id = try!(read_numeric_data(rd));

    match read_full(rd, buf) {
        Ok(n) if n == buf.len() => Ok(id),
        Ok(..)   => Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)),
        Err(err) => Err(ValueReadError::InvalidDataRead(ReadError::Io(err))),
    }
}

/// Copies the contents of a reader into a buffer until fully filled.
///
/// This function will continuously read data from `rd` and then write it into `buf` in a streaming
/// fashion until `rd` returns EOF.
///
/// On success the total number of bytes that were copied from `rd` to `buf` is returned. Note, that
/// reaching EOF is not treated as error.
///
/// # Errors
///
/// This function will return an error immediately if any call to `read` returns an error. All
/// instances of `ErrorKind::Interrupted` are handled by this function and the underlying operation
/// is retried.
fn read_full<R: Read>(rd: &mut R, buf: &mut [u8]) -> Result<usize, io::Error> {
    // TODO: Maybe move this helper function into an independent module
    let mut nread = 0usize;

    while nread < buf.len() {
        match rd.read(&mut buf[nread..]) {
            Ok(0) => return Ok(nread),
            Ok(n) => nread += n,
            Err(ref err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err)
        }
    }

    Ok(nread)
}

#[derive(Debug, PartialEq)]
pub struct ExtMeta {
    pub typeid: i8,
    pub size: u32,
}

/// Unstable: docs, errors
// NOTE: EINTR safe.
pub fn read_ext_meta<R>(rd: &mut R) -> Result<ExtMeta, ValueReadError>
    where R: Read
{
    let size = match try!(read_marker(rd)) {
        Marker::FixExt1  => 1,
        Marker::FixExt2  => 2,
        Marker::FixExt4  => 4,
        Marker::FixExt8  => 8,
        Marker::FixExt16 => 16,
        Marker::Ext8     => try!(read_numeric_data::<R, u8>(rd))  as u32,
        Marker::Ext16    => try!(read_numeric_data::<R, u16>(rd)) as u32,
        Marker::Ext32    => try!(read_numeric_data(rd)),
        marker           => return Err(ValueReadError::TypeMismatch(marker))
    };

    let typeid = try!(read_numeric_data(rd));
    let meta = ExtMeta { typeid: typeid, size: size };

    Ok(meta)
}

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

pub mod serialize {

use std::convert::From;
use std::fmt;
use std::io::Read;
use std::result;

use serialize;

use super::super::Marker;
use super::{
    ReadError,
    FixedValueReadError,
    ValueReadError,
    DecodeStringError,
    read_nil,
    read_bool,
    read_u8_fit,
    read_u16_fit,
    read_u32_fit,
    read_u64_fit,
    read_i8_fit,
    read_i16_fit,
    read_i32_fit,
    read_i64_fit,
    read_f32,
    read_f64,
    read_str_len,
    read_str_data,
    read_array_size,
    read_map_size,
};

/// Unstable: docs; incomplete
#[derive(Debug)]
pub enum Error {
    /// The actual value type isn't equal with the expected one.
    TypeMismatch(Marker),
    InvalidMarkerRead(ReadError),
    InvalidDataRead(ReadError),
    LengthMismatch(u32),
    /// Uncategorized error.
    Uncategorized(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "error while decoding value" }

    fn cause(&self) -> Option<&::std::error::Error> {
        use self::Error::*;
        match *self {
            TypeMismatch(_) => None,
            InvalidMarkerRead(ref err) => Some(err),
            InvalidDataRead(ref err) => Some(err),
            LengthMismatch(_) => None,
            Uncategorized(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl From<FixedValueReadError> for Error {
    fn from(err: FixedValueReadError) -> Error {
        match err {
            FixedValueReadError::UnexpectedEOF => Error::InvalidMarkerRead(ReadError::UnexpectedEOF),
            FixedValueReadError::Io(err) => Error::InvalidMarkerRead(ReadError::Io(err)),
            FixedValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Error {
        match err {
            ValueReadError::TypeMismatch(marker)   => Error::TypeMismatch(marker),
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err)   => Error::InvalidDataRead(err),
        }
    }
}

/// Unstable: docs; incomplete
impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(..) => Error::Uncategorized("InvalidDataRead".to_string()),
            DecodeStringError::TypeMismatch(..) => Error::Uncategorized("TypeMismatch".to_string()),
            DecodeStringError::BufferSizeTooSmall(..) => Error::Uncategorized("BufferSizeTooSmall".to_string()),
            DecodeStringError::InvalidDataCopy(..) => Error::Uncategorized("InvalidDataCopy".to_string()),
            DecodeStringError::InvalidUtf8(..) => Error::Uncategorized("InvalidUtf8".to_string()),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub struct Decoder<R: Read> {
    rd: R,
}

impl<R: Read> Decoder<R> {
    // TODO: Docs.
    pub fn new(rd: R) -> Decoder<R> {
        Decoder {
            rd: rd
        }
    }

    /// Gets a reference to the underlying reader in this decoder.
    pub fn get_ref(&self) -> &R {
        &self.rd
    }

    /// Gets a mutable reference to the underlying reader in this decoder.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.rd
    }

    /// Consumes this decoder returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.rd
    }
}

/// Unstable: docs; examples; incomplete
impl<R: Read> serialize::Decoder for Decoder<R> {
    type Error = Error;

    fn read_nil(&mut self) -> Result<()> {
        Ok(try!(read_nil(&mut self.rd)))
    }

    fn read_bool(&mut self) -> Result<bool> {
        Ok(try!(read_bool(&mut self.rd)))
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(try!(read_u8_fit(&mut self.rd)))
    }

    fn read_u16(&mut self) -> Result<u16> {
        Ok(try!(read_u16_fit(&mut self.rd)))
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(try!(read_u32_fit(&mut self.rd)))
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(try!(read_u64_fit(&mut self.rd)))
    }

    /// TODO: Doesn't look safe.
    fn read_usize(&mut self) -> Result<usize> {
        let v = try!(self.read_u64());
        Ok(v as usize)
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(try!(read_i8_fit(&mut self.rd)))
    }

    fn read_i16(&mut self) -> Result<i16> {
        Ok(try!(read_i16_fit(&mut self.rd)))
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(try!(read_i32_fit(&mut self.rd)))
    }

    fn read_i64(&mut self) -> Result<i64> {
        Ok(try!(read_i64_fit(&mut self.rd)))
    }

    /// TODO: Doesn't look safe.
    fn read_isize(&mut self) -> Result<isize> {
        Ok(try!(self.read_i64()) as isize)
    }

    fn read_f32(&mut self) -> Result<f32> {
        Ok(try!(read_f32(&mut self.rd)))
    }

    fn read_f64(&mut self) -> Result<f64> {
        Ok(try!(read_f64(&mut self.rd)))
    }

    fn read_char(&mut self) -> Result<char> {
        let mut res = try!(self.read_str());
        if res.len() == 1 {
            Ok(res.pop().unwrap())
        } else {
            Err(self.error("length mismatch"))
        }
    }

    fn read_str(&mut self) -> Result<String> {
        let len = try!(read_str_len(&mut self.rd));

        let mut buf: Vec<u8> = (0..len).map(|_| 0u8).collect();

        Ok(try!(read_str_data(&mut self.rd, len, &mut buf[..])).to_string())
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        let len = try!(read_array_size(&mut self.rd));
        if len == 2 {
            f(self)
        } else {
            Err(self.error("sequence length mismatch"))
        }
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str], mut f: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        let id = try!(self.read_usize());

        if id < names.len() {
            try!(read_array_size(&mut self.rd));

            f(self, id)
        } else {
            Err(self.error("variant type overflow"))
        }
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        self.read_enum_variant(names, f)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_struct<T, F>(&mut self, _name: &str, len: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        self.read_tuple(len, f)
    }

    fn read_struct_field<T, F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, len: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        let actual = try!(read_array_size(&mut self.rd));

        if len == actual as usize {
            f(self)
        } else {
            Err(Error::LengthMismatch(actual))
        }
    }

    // In case of MessagePack don't care about argument indexing.
    fn read_tuple_arg<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _name: &str, _len: usize, _f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T> { unimplemented!() }
    fn read_tuple_struct_arg<T, F>(&mut self, _idx: usize, _f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T> { unimplemented!() }

    /// We treat Value::Null as None.
    fn read_option<T, F>(&mut self, mut f: F) -> Result<T>
        where F: FnMut(&mut Self, bool) -> Result<T>
    {
        // Primarily try to read optimisticly.
        match f(self, true) {
            Ok(val) => Ok(val),
            Err(Error::TypeMismatch(Marker::Null)) => f(self, false),
            Err(err) => Err(err)
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        let len = try!(read_array_size(&mut self.rd)) as usize;

        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        let len = try!(read_map_size(&mut self.rd)) as usize;

        f(self, len)
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> Error {
        Error::Uncategorized(err.to_string())
    }
}

}

pub mod serde {

use std::convert::From;
use std::fmt;
use std::io::Read;
use std::result;

use serde;

use super::super::Marker;
use super::{
    ReadError,
    FixedValueReadError,
    MarkerReadError,
    ValueReadError,
    DecodeStringError,
    read_numeric_data,
    read_str_data,
    read_marker,
    read_full,
};

/// Unstable: docs; incomplete
#[derive(Debug)]
pub enum Error {
    /// The actual value type isn't equal with the expected one.
    TypeMismatch(Marker),
    InvalidMarkerRead(ReadError),
    InvalidDataRead(ReadError),
    LengthMismatch(u32),
    /// Uncategorized error.
    Uncategorized(String),
    Syntax(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "error while decoding value" }

    fn cause(&self) -> Option<&::std::error::Error> {
        use self::Error::*;
        match *self {
            TypeMismatch(_) => None,
            InvalidMarkerRead(ref err) => Some(err),
            InvalidDataRead(ref err) => Some(err),
            LengthMismatch(_) => None,
            Uncategorized(_) => None,
            Syntax(_) => None,
        }
    }
}

impl serde::de::Error for Error {
    fn syntax(msg: &str) -> Error {
        Error::Syntax(format!("syntax error: {}", msg))
    }

    fn length_mismatch(len: usize) -> Error {
        Error::LengthMismatch(len as u32)
    }

    fn type_mismatch(ty: serde::de::Type) -> Error {
        match ty {
            serde::de::Type::Bool => Error::TypeMismatch(Marker::True),
            serde::de::Type::Usize => Error::TypeMismatch(Marker::FixPos(0)),
            serde::de::Type::U8 => Error::TypeMismatch(Marker::U8),
            serde::de::Type::U16 => Error::TypeMismatch(Marker::U16),
            serde::de::Type::U32 => Error::TypeMismatch(Marker::U32),
            serde::de::Type::U64 => Error::TypeMismatch(Marker::U64),
            serde::de::Type::Isize => Error::TypeMismatch(Marker::FixNeg(0)),
            serde::de::Type::I8 => Error::TypeMismatch(Marker::I8),
            serde::de::Type::I16 => Error::TypeMismatch(Marker::I16),
            serde::de::Type::I32 => Error::TypeMismatch(Marker::I32),
            serde::de::Type::I64 => Error::TypeMismatch(Marker::I64),
            serde::de::Type::F32 => Error::TypeMismatch(Marker::F32),
            serde::de::Type::F64 => Error::TypeMismatch(Marker::F64),
            serde::de::Type::Char => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::Str => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::String => Error::TypeMismatch(Marker::Str32),
            serde::de::Type::Unit => Error::TypeMismatch(Marker::Null),
            serde::de::Type::Option => Error::TypeMismatch(Marker::True),
            serde::de::Type::Seq => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Map => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::UnitStruct => Error::TypeMismatch(Marker::Null),
            serde::de::Type::NewtypeStruct => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::TupleStruct => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Struct => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::Tuple => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Enum => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::StructVariant => Error::TypeMismatch(Marker::Map32),
            serde::de::Type::TupleVariant => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::UnitVariant => Error::TypeMismatch(Marker::Array32),
            serde::de::Type::Bytes => Error::TypeMismatch(Marker::Array32),
        }
    }

    fn end_of_stream() -> Error {
        Error::Uncategorized("end of stream".to_string())
    }

    fn missing_field(_field: &str) -> Error {
        Error::Uncategorized("missing field".to_string())
    }

    fn unknown_field(_field: &str) -> Error {
        Error::Uncategorized("unknown field".to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl From<FixedValueReadError> for Error {
    fn from(err: FixedValueReadError) -> Error {
        match err {
            FixedValueReadError::UnexpectedEOF => Error::InvalidMarkerRead(ReadError::UnexpectedEOF),
            FixedValueReadError::Io(err) => Error::InvalidMarkerRead(ReadError::Io(err)),
            FixedValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Error {
        match err {
            ValueReadError::TypeMismatch(marker)   => Error::TypeMismatch(marker),
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err)   => Error::InvalidDataRead(err),
        }
    }
}

/// Unstable: docs; incomplete
impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(..) => Error::Uncategorized("InvalidDataRead".to_string()),
            DecodeStringError::TypeMismatch(..) => Error::Uncategorized("TypeMismatch".to_string()),
            DecodeStringError::BufferSizeTooSmall(..) => Error::Uncategorized("BufferSizeTooSmall".to_string()),
            DecodeStringError::InvalidDataCopy(..) => Error::Uncategorized("InvalidDataCopy".to_string()),
            DecodeStringError::InvalidUtf8(..) => Error::Uncategorized("InvalidUtf8".to_string()),
        }
    }
}

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        Error::InvalidMarkerRead(From::from(err))
    }
}

pub type Result<T> = result::Result<T, Error>;

/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub struct Deserializer<R: Read> {
    rd: R,
    buf: Vec<u8>,
}

impl<R: Read> Deserializer<R> {
    // TODO: Docs.
    pub fn new(rd: R) -> Deserializer<R> {
        Deserializer {
            rd: rd,
            buf: Vec::new(),
        }
    }

    fn read_str<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        self.buf.clear();
        self.buf.extend((0..len).map(|_| 0));
        visitor.visit_str(try!(read_str_data(&mut self.rd, len, &mut self.buf[..])))
    }

    fn read_array<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_seq(SeqVisitor {
            deserializer: self,
            len: len,
            actual: len,
        })
    }

    fn read_map<V>(&mut self, len: u32, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        visitor.visit_map(MapVisitor {
            deserializer: self,
            len: len,
            actual: len,
        })
    }

    fn read_bin_data<V>(&mut self, len: usize, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        self.buf.clear();
        self.buf.extend((0..len).map(|_| 0));

        match read_full(&mut self.rd, &mut self.buf[..]) {
            Ok(n) if n == self.buf.len() => (),
            Ok(..)   => return Err(Error::InvalidDataRead(ReadError::UnexpectedEOF)),
            Err(err) => return Err(Error::InvalidDataRead(ReadError::Io(err))),
        }

        visitor.visit_bytes(&mut self.buf[..])
    }

    // TODO: Add an ability to borrow underlying reader and to destruct this decoder.
}

/// Unstable: docs; examples; incomplete
impl<R: Read> serde::Deserializer for Deserializer<R> {
    type Error = Error;

    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor
    {
        match try!(read_marker(&mut self.rd)) {
            Marker::Null => visitor.visit_unit(),
            Marker::True => visitor.visit_bool(true),
            Marker::False => visitor.visit_bool(false),
            Marker::FixPos(val) => visitor.visit_u8(val),
            Marker::FixNeg(val) => visitor.visit_i8(val),
            Marker::U8 => visitor.visit_u8(try!(read_numeric_data(&mut self.rd))),
            Marker::U16 => visitor.visit_u16(try!(read_numeric_data(&mut self.rd))),
            Marker::U32 => visitor.visit_u32(try!(read_numeric_data(&mut self.rd))),
            Marker::U64 => visitor.visit_u64(try!(read_numeric_data(&mut self.rd))),
            Marker::I8 => visitor.visit_i8(try!(read_numeric_data(&mut self.rd))),
            Marker::I16 => visitor.visit_i16(try!(read_numeric_data(&mut self.rd))),
            Marker::I32 => visitor.visit_i32(try!(read_numeric_data(&mut self.rd))),
            Marker::I64 => visitor.visit_i64(try!(read_numeric_data(&mut self.rd))),
            Marker::F32 => visitor.visit_f32(try!(read_numeric_data(&mut self.rd))),
            Marker::F64 => visitor.visit_f64(try!(read_numeric_data(&mut self.rd))),
            Marker::FixStr(len) => self.read_str(len as u32, visitor),
            Marker::Str8 => {
                let len: u8 = try!(read_numeric_data(&mut self.rd));
                self.read_str(len as u32, visitor)
            }
            Marker::Str16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_str(len as u32, visitor)
            }
            Marker::Str32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_str(len, visitor)
            }
            Marker::FixArray(len) => {
                self.read_array(len as u32, visitor)
            }
            Marker::Array16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_array(len as u32, visitor)
            }
            Marker::Array32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_array(len, visitor)
            }
            Marker::FixMap(len) => {
                self.read_map(len as u32, visitor)
            }
            Marker::Map16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_map(len as u32, visitor)
            }
            Marker::Map32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_map(len, visitor)
            }
            Marker::Bin8 => {
                let len: u8 = try!(read_numeric_data(&mut self.rd));
                self.read_bin_data(len as usize, visitor)
            }
            Marker::Bin16 => {
                let len: u16 = try!(read_numeric_data(&mut self.rd));
                self.read_bin_data(len as usize, visitor)
            }
            Marker::Bin32 => {
                let len: u32 = try!(read_numeric_data(&mut self.rd));
                self.read_bin_data(len as usize, visitor)
            }
            Marker::Reserved => Err(Error::TypeMismatch(Marker::Reserved)),
            // TODO: Make something with exts.
            marker => Err(From::from(FixedValueReadError::TypeMismatch(marker))),
        }
    }

    /// We treat Value::Null as None.
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor,
    {
        // Primarily try to read optimisticly.
        match visitor.visit_some(self) {
            Ok(val) => Ok(val),
            Err(Error::TypeMismatch(Marker::Null)) => visitor.visit_none(),
            Err(err) => Err(err)
        }
    }
}

struct SeqVisitor<'a, R: Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    len: u32,
    actual: u32,
}

impl<'a, R: Read + 'a> serde::de::SeqVisitor for SeqVisitor<'a, R> {
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>>
        where T: serde::de::Deserialize,
    {
        if self.len > 0 {
            self.len -= 1;
            let value = try!(serde::Deserialize::deserialize(self.deserializer));
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn end(&mut self) -> Result<()> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.actual))
        }
    }
}

struct MapVisitor<'a, R: Read + 'a> {
    deserializer: &'a mut Deserializer<R>,
    len: u32,
    actual: u32,
}

impl<'a, R: Read + 'a> serde::de::MapVisitor for MapVisitor<'a, R> {
    type Error = Error;

    fn visit_key<K>(&mut self) -> Result<Option<K>>
        where K: serde::de::Deserialize,
    {
        if self.len > 0 {
            self.len -= 1;
            let key = try!(serde::Deserialize::deserialize(self.deserializer));
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    fn visit_value<V>(&mut self) -> Result<V>
        where V: serde::de::Deserialize,
    {
        let value = try!(serde::Deserialize::deserialize(self.deserializer));
        Ok(value)
    }

    fn end(&mut self) -> Result<()> {
        if self.len == 0 {
            Ok(())
        } else {
            Err(Error::LengthMismatch(self.actual))
        }
    }
}

}

pub use self::value::read_value;
