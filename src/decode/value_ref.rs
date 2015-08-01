//! This module is UNSTABLE, the reason is - recently added.

use std::convert::From;
use std::error;
use std::fmt;
use std::io::{Cursor, Read};
use std::str::{from_utf8, Utf8Error};

use super::{read_marker};
use super::{
    ReadError,
    MarkerReadError,
};
use super::{BigEndianRead};

use super::super::init::Marker;
use super::super::value::{Float, Integer, ValueRef};

trait ToUnsigned {
    fn from(v: Self) -> Option<usize> where Self: Sized;
}

impl ToUnsigned for u8 {
    fn from(v: u8) -> Option<usize> {
        // Impossible to panic, since u8 always fits in usize.
        Some(v as usize)
    }
}

impl ToUnsigned for u16 {
    fn from(v: u16) -> Option<usize> {
        // TODO: This can overflow on 8-bit systems.
        Some(v as usize)
    }
}

impl ToUnsigned for u32 {
    fn from(v: u32) -> Option<usize> {
        // TODO: This can overflow on 8- and 16-bit systems.
        Some(v as usize)
    }
}

#[derive(Debug)]
pub enum Error<'r> {
    /// Failed to read the type marker value.
    InvalidMarkerRead(ReadError),
    /// Failed to read string/array/map size.
    InvalidLengthRead(ReadError),
    /// Failed to read packed non-marker data.
    InvalidDataRead(ReadError),
    /// Failed to cast the length read to machine size.
    InvalidLengthSize,
    /// Failed to interpret a byte slice as a UTF-8 string.
    ///
    /// Contains untouched bytearray with the underlying decoding error.
    InvalidUtf8(&'r [u8], Utf8Error),
    /// Failed to read ext type.
    InvalidExtTypeRead(ReadError),
    /// Using Reserved type found.
    TypeMismatch,
}

impl<'r> error::Error for Error<'r> {
    fn description(&self) -> &str {
        match self {
            &Error::InvalidMarkerRead(..) => "failed to read the type marker value",
            &Error::InvalidLengthRead(..) => "failed to read string/array/map size",
            &Error::InvalidDataRead(..) => "failed to read packed non-marker data",
            &Error::InvalidLengthSize => "failed to cast the length read to machine size",
            &Error::InvalidUtf8(..) => "failed to interpret a byte slice as a UTF-8 string",
            &Error::InvalidExtTypeRead(..) => "failed to read ext type",
            &Error::TypeMismatch => "using Reserved type found",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error::InvalidMarkerRead(ref err) => Some(err),
            &Error::InvalidLengthRead(ref err) => Some(err),
            &Error::InvalidDataRead(ref err) => Some(err),
            &Error::InvalidLengthSize => None,
            &Error::InvalidUtf8(_, ref err) => Some(err),
            &Error::InvalidExtTypeRead(ref err) => Some(err),
            &Error::TypeMismatch => None,
        }
    }
}

impl<'r> fmt::Display for Error<'r> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use std::error::Error;
        self.description().fmt(f)
    }
}

impl<'r> From<MarkerReadError> for Error<'r> {
    fn from(err: MarkerReadError) -> Error<'r> {
        Error::InvalidMarkerRead(From::from(err))
    }
}

fn read_len<R, D>(rd: &mut R) -> Result<D, ReadError>
    where R: Read,
          D: BigEndianRead
{
    D::read(rd).map_err(From::from)
}

fn read_num<'a, R, D>(mut rd: &mut R) -> Result<D, Error<'a>>
    where R: BufRead<'a>,
          D: BigEndianRead
{
    D::read(&mut rd).map_err(|err| Error::InvalidDataRead(From::from(err)))
}

fn read_str<'a, R>(rd: &mut R, len: usize) -> Result<&'a str, Error<'a>>
    where R: BufRead<'a>
{
    let buf = try!(read_bin(rd, len));

    // Try to decode sliced buffer as UTF-8.
    let res = try!(from_utf8(buf).map_err(|err| Error::InvalidUtf8(buf, err)));

    Ok(res)
}

fn read_bin<'a, R>(rd: &mut R, len: usize) -> Result<&'a [u8], Error<'a>>
    where R: BufRead<'a>
{
    let buf = rd.fill_buf();

    if len > buf.len() {
        return Err(Error::InvalidDataRead(ReadError::UnexpectedEOF));
    }

    // Take a slice.
    let buf = &buf[..len];
    rd.consume(len);

    Ok(buf)
}

// Helper function that reads a single byte from the given `Read` and interpret it as an Ext type.
fn read_ext_type<R>(rd: &mut R) -> Result<i8, ReadError>
    where R: Read
{
    i8::read(rd).map_err(From::from)
}

fn read_ext<'a, R>(mut rd: &mut R, len: usize) -> Result<(i8, &'a [u8]), Error<'a>>
    where R: BufRead<'a>
{
    let ty  = try!(read_ext_type(&mut rd).map_err(|err| Error::InvalidExtTypeRead(err)));
    let buf = try!(read_bin(rd, len));

    Ok((ty, buf))
}

#[inline]
fn read_str_value<'a, R, U>(rd: &mut R, len: U) -> Result<ValueRef<'a>, Error<'a>>
    where R: BufRead<'a>,
          U: ToUnsigned
{
    let len = try!(U::from(len).ok_or(Error::InvalidLengthSize));
    let res = try!(read_str(rd, len));

    Ok(ValueRef::String(res))
}

#[inline]
fn read_bin_value<'a, R, U>(rd: &mut R, len: U) -> Result<ValueRef<'a>, Error<'a>>
    where R: BufRead<'a>,
          U: ToUnsigned
{
    let len = try!(U::from(len).ok_or(Error::InvalidLengthSize));
    let res = try!(read_bin(rd, len));

    Ok(ValueRef::Binary(res))
}

#[inline]
fn read_ext_value<'a, R, U>(mut rd: &mut R, len: U) -> Result<ValueRef<'a>, Error<'a>>
    where R: BufRead<'a>,
          U: ToUnsigned
{
    let len = try!(U::from(len).ok_or(Error::InvalidLengthSize));
    let (ty, buf) = try!(read_ext(rd, len));

    Ok(ValueRef::Ext(ty, buf))
}

#[inline]
fn read_array_value<'a, R, U>(rd: &mut R, len: U) -> Result<ValueRef<'a>, Error<'a>>
    where R: BufRead<'a>,
          U: ToUnsigned
{
    let len = try!(U::from(len).ok_or(Error::InvalidLengthSize));
    let vec = try!(read_array(rd, len));

    Ok(ValueRef::Array(vec))
}

#[inline]
fn read_map_value<'a, R, U>(rd: &mut R, len: U) -> Result<ValueRef<'a>, Error<'a>>
    where R: BufRead<'a>,
          U: ToUnsigned
{
    let len = try!(U::from(len).ok_or(Error::InvalidLengthSize));
    let map = try!(read_map(rd, len));

    Ok(ValueRef::Map(map))
}

fn read_array<'a, R>(rd: &mut R, len: usize) -> Result<Vec<ValueRef<'a>>, Error<'a>>
    where R: BufRead<'a>
{
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
        let val = try!(read_value_ref(rd));

        vec.push(val);
    }

    Ok(vec)
}

fn read_map<'a, R>(rd: &mut R, len: usize) -> Result<Vec<(ValueRef<'a>, ValueRef<'a>)>, Error<'a>>
    where R: BufRead<'a>
{
    let mut vec = Vec::with_capacity(len);

    for _ in 0..len {
        let key = try!(read_value_ref(rd));
        let val = try!(read_value_ref(rd));

        vec.push((key, val));
    }

    Ok(vec)
}

/// A BufRead is a type of Reader which has an internal buffer.
///
/// This magic trait acts like a standard BufRead but unlike the standard this has an explicit
/// internal buffer lifetime, which allows to borrow from underlying buffer while consuming bytes.
// TODO: Rename to BorrowRead to avoid naming collisions.
pub trait BufRead<'a>: Read {
    /// Returns the buffer contents.
    ///
    /// This function is a lower-level call. It needs to be paired with the consume method to
    /// function properly. When calling this method, none of the contents will be "read" in the
    /// sense that later calling read may return the same contents. As such, consume must be called
    /// with the number of bytes that are consumed from this buffer to ensure that the bytes are
    /// never returned twice.
    ///
    /// An empty buffer returned indicates that the stream has reached EOF.
    fn fill_buf(&self) -> &'a [u8];

    /// Tells this buffer that len bytes have been consumed from the buffer, so they should no
    /// longer be returned in calls to read.
    fn consume(&mut self, len: usize);
}

impl<'a> BufRead<'a> for &'a [u8] {
    fn fill_buf(&self) -> &'a [u8] {
        self
    }

    fn consume(&mut self, len: usize) {
        *self = &(*self)[len..];
    }
}

/// Useful when you want to know how much bytes has been consumed during ValueRef decoding.
impl<'a> BufRead<'a> for Cursor<&'a [u8]> {
    fn fill_buf(&self) -> &'a [u8] {
        use std::cmp;

        let len = cmp::min(self.position(), self.get_ref().len() as u64);
        &self.get_ref()[len as usize..]
    }

    fn consume(&mut self, len: usize) {
        let pos = self.position();
        self.set_position(pos + len as u64);
    }
}

/// Attempts to read the data from the given reader until either a complete MessagePack value
/// decoded or an error detected.
///
/// Returns either a non-owning `ValueRef`, which borrows the buffer from the given reader or an
/// error.
///
/// The reader should meet the requirement of a special `BorrowRead` trait, which allows to mutate
/// itself but permits to mutate the buffer it contains. It allows to perform a completely
/// zero-copy reading without a data loss fear in case of an error.
///
/// Currently only two types fit in this requirement: `&[u8]` and `Cursor<&[u8]>`. Using Cursor is
/// helpful, when you need to know how exactly many bytes the decoded ValueRef consumes. A `Vec<u8>`
/// type doesn't fit in the `BorrowRead` requirement, because its mut reference can mutate the
/// underlying buffer - use `Vec::as_slice()` if you need to decode a value from the vector.
///
/// # Errors
///
/// Returns an `Error` value if unable to continue the decoding operation either because of read
/// failure or any other circumstances. See `Error` documentation for more information.
///
/// # Examples
/// ```
/// use rmp::ValueRef;
/// use rmp::decode::value_ref::read_value_ref;
///
/// let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
/// let mut rd = &buf[..];
///
/// assert_eq!(ValueRef::String("le message"), read_value_ref(&mut rd).unwrap());
/// ```
pub fn read_value_ref<'a, R>(rd: &mut R) -> Result<ValueRef<'a>, Error<'a>>
    where R: BufRead<'a>
{
    let mut rd = rd;

    // Reading the marker involves either 1 byte read or nothing. On success consumes strictly
    // 1 byte from the `rd`.
    let marker = try!(read_marker(rd));

    let val = match marker {
        Marker::Null => ValueRef::Nil,
        Marker::True => ValueRef::Boolean(true),
        Marker::False => ValueRef::Boolean(false),
        Marker::PositiveFixnum(val) => {
            ValueRef::Integer(Integer::U64(val as u64))
        }
        Marker::U8 => {
            let val: u8 = try!(read_num(rd));
            ValueRef::Integer(Integer::U64(val as u64))
        }
        Marker::U16 => {
            let val: u16 = try!(read_num(rd));
            ValueRef::Integer(Integer::U64(val as u64))
        }
        Marker::U32 => {
            let val: u32 = try!(read_num(rd));
            ValueRef::Integer(Integer::U64(val as u64))
        }
        Marker::U64 => {
            let val: u64 = try!(read_num(rd));
            ValueRef::Integer(Integer::U64(val))
        }
        Marker::NegativeFixnum(val) => {
            ValueRef::Integer(Integer::I64(val as i64))
        }
        Marker::I8 => {
            let val: i8 = try!(read_num(rd));
            ValueRef::Integer(Integer::I64(val as i64))
        }
        Marker::I16 => {
            let val: i16 = try!(read_num(rd));
            ValueRef::Integer(Integer::I64(val as i64))
        }
        Marker::I32 => {
            let val: i32 = try!(read_num(rd));
            ValueRef::Integer(Integer::I64(val as i64))
        }
        Marker::I64 => {
            let val: i64 = try!(read_num(rd));
            ValueRef::Integer(Integer::I64(val))
        }
        Marker::F32 => {
            let val: f32 = try!(read_num(rd));
            ValueRef::Float(Float::F32(val))
        }
        Marker::F64 => {
            let val: f64 = try!(read_num(rd));
            ValueRef::Float(Float::F64(val))
        }
        Marker::FixedString(len) => {
            try!(read_str_value(rd, len))
        }
        Marker::Str8 => {
            let len: u8 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_str_value(rd, len))
        }
        Marker::Str16 => {
            let len: u16 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_str_value(rd, len))
        }
        Marker::Str32 => {
            let len: u32 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_str_value(rd, len))
        }
        Marker::Bin8 => {
            let len: u8 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_bin_value(rd, len))
        }
        Marker::Bin16 => {
            let len: u16 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_bin_value(rd, len))
        }
        Marker::Bin32 => {
            let len: u32 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_bin_value(rd, len))
        }
        Marker::FixedArray(len) => {
            try!(read_array_value(rd, len))
        }
        Marker::Array16 => {
            let len: u16 = try!(read_len(&mut rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_array_value(rd, len))
        }
        Marker::Array32 => {
            let len: u32 = try!(read_len(&mut rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_array_value(rd, len))
        }
        Marker::FixedMap(len) => {
            try!(read_map_value(rd, len))
        }
        Marker::Map16 => {
            let len: u16 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_map_value(rd, len))
        }
        Marker::Map32 => {
            let len: u32 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_map_value(rd, len))
        }
        Marker::FixExt1 => {
            try!(read_ext_value(rd, 1u8))
        }
        Marker::FixExt2 => {
            try!(read_ext_value(rd, 2u8))
        }
        Marker::FixExt4 => {
            try!(read_ext_value(rd, 4u8))
        }
        Marker::FixExt8 => {
            try!(read_ext_value(rd, 8u8))
        }
        Marker::FixExt16 => {
            try!(read_ext_value(rd, 16u8))
        }
        Marker::Ext8 => {
            let len: u8 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_ext_value(rd, len))
        }
        Marker::Ext16 => {
            let len: u16 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_ext_value(rd, len))
        }
        Marker::Ext32 => {
            let len: u32 = try!(read_len(rd).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_ext_value(rd, len))
        }
        Marker::Reserved => return Err(Error::TypeMismatch),
    };

    Ok(val)
}
