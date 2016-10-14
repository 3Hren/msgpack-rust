//! Provides various functions and structs for MessagePack decoding.
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
        pub fn $name<R>(rd: &mut R) -> Result<$ty, ValueReadError>
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

pub fn read_str_data<'r, R>(rd: &mut R, len: u32, buf: &'r mut[u8]) -> Result<&'r str, DecodeStringError<'r>>
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

/// Attempts to read up to 5 bytes from the given reader and to decode them as Binary array length.
///
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

/// Attempts to read some bytes from the given slice until a complete Binary message is decoded,
/// returning a borrowed slice with the data.
///
/// This includes reading both length and the data itself.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
///
/// # Unstable
///
/// This function is unstable, moreover I have a plan to drop it.
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
// TODO Make private again (pub is required for serde crate).
pub fn read_full<R: Read>(rd: &mut R, buf: &mut [u8]) -> Result<usize, io::Error> {
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

pub use self::value::read_value;
