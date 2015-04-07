use std::convert::From;
use std::io;
use std::io::{Cursor, Read};
use std::num::FromPrimitive;
use std::result;
use std::str::from_utf8;

use byteorder::{self, ReadBytesExt};

use super::{Marker, Error, MarkerError, ReadError, Result, Integer, DecodeStringError};

fn read_marker<R>(rd: &mut R) -> Result<Marker>
    where R: Read
{
    match rd.read_u8() {
        Ok(val) => {
            match FromPrimitive::from_u8(val) {
                Some(marker) => Ok(marker),
                None         => Err(Error::InvalidMarker(MarkerError::Unexpected(val))),
            }
        }
        Err(err) => Err(Error::InvalidMarkerRead(From::from(err))),
    }
}

/// Tries to decode a nil value from the reader.
#[stable(since = "0.1.0")]
pub fn read_nil<R>(rd: &mut R) -> Result<()>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::Null => Ok(()),
        _            => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

/// Tries to decode a bool value from the reader.
#[stable(since = "0.1.0")]
pub fn read_bool<R>(rd: &mut R) -> Result<bool>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::True  => Ok(true),
        Marker::False => Ok(false),
        _             => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

/// Tries to decode an exactly positive fixnum from the reader.
#[stable(since = "0.1.0")]
pub fn read_pfix<R>(rd: &mut R) -> Result<u8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::PositiveFixnum(val) => Ok(val),
        _                           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to decode an exactly negative fixnum from the reader.
#[stable(since = "0.1.0")]
pub fn read_nfix<R>(rd: &mut R) -> Result<i8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::NegativeFixnum(val) => Ok(val),
        _                           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i8 value from the reader.
#[stable(since = "0.1.0")]
pub fn read_i8<R>(rd: &mut R) -> Result<i8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I8 => Ok(try!(read_data_i8(rd))),
        _          => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i16 value from the reader.
#[stable(since = "0.1.0")]
pub fn read_i16<R>(rd: &mut R) -> Result<i16>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I16 => Ok(try!(read_data_i16(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i32 value from the reader.
#[stable(since = "0.1.0")]
pub fn read_i32<R>(rd: &mut R) -> Result<i32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I32 => Ok(try!(read_data_i32(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i64 value from the reader.
#[stable(since = "0.1.0")]
pub fn read_i64<R>(rd: &mut R) -> Result<i64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I64 => Ok(try!(read_data_i64(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read exactly 2 bytes from the reader and decode them as u8.
#[stable(since = "0.1.0")]
pub fn read_u8<R>(rd: &mut R) -> Result<u8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::U8 => Ok(try!(read_data_u8(rd))),
        _          => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read exactly 3 bytes from the reader and decode them as u16.
#[unstable(reason = "more docs")]
pub fn read_u16<R>(rd: &mut R) -> Result<u16>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::U16 => Ok(try!(read_data_u16(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read exactly 5 bytes from the reader and decode them as u32.
#[unstable(reason = "more docs")]
pub fn read_u32<R>(rd: &mut R) -> Result<u32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::U32 => Ok(try!(read_data_u32(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read exactly 5 bytes from the reader and decode them as u64
#[unstable(reason = "more docs")]
pub fn read_u64<R>(rd: &mut R) -> Result<u64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::U64 => Ok(try!(read_data_u64(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

macro_rules! make_read_data_fn {
    (deduce, $reader:ident, $decoder:ident, 0)
        => ($reader.$decoder(););
    (deduce, $reader:ident, $decoder:ident, 1)
        => ($reader.$decoder::<byteorder::BigEndian>(););
    (gen, $t:ty, $d:tt, $name:ident, $decoder:ident) => {
        fn $name<R>(rd: &mut R) -> Result<$t>
            where R: Read
        {
            match make_read_data_fn!(deduce, rd, $decoder, $d) {
                Ok(data) => Ok(data),
                Err(err) => Err(Error::InvalidDataRead(From::from(err))),
            }
        }
    };
    (u8,    $name:ident, $decoder:ident) => (make_read_data_fn!(gen, u8, 0, $name, $decoder););
    (i8,    $name:ident, $decoder:ident) => (make_read_data_fn!(gen, i8, 0, $name, $decoder););
    ($t:ty, $name:ident, $decoder:ident) => (make_read_data_fn!(gen, $t, 1, $name, $decoder););
}

make_read_data_fn!(u8,  read_data_u8,  read_u8);
make_read_data_fn!(u16, read_data_u16, read_u16);
make_read_data_fn!(u32, read_data_u32, read_u32);
make_read_data_fn!(u64, read_data_u64, read_u64);
make_read_data_fn!(i8,  read_data_i8,  read_i8);
make_read_data_fn!(i16, read_data_i16, read_i16);
make_read_data_fn!(i32, read_data_i32, read_i32);
make_read_data_fn!(i64, read_data_i64, read_i64);
make_read_data_fn!(f32, read_data_f32, read_f32);
make_read_data_fn!(f64, read_data_f64, read_f64);

/// Tries to read up to 9 bytes from the reader (1 for marker and up to 8 for data) and interpret
/// them as a big-endian u64.
///
/// The function tries to decode only unsigned integer values that are always non-negative.
#[unstable(reason = "not sure about name")]
pub fn read_u64_loosely<R>(rd: &mut R) -> Result<u64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::PositiveFixnum(val) => Ok(val as u64),
        Marker::U8  => Ok(try!(read_data_u8(rd))  as u64),
        Marker::U16 => Ok(try!(read_data_u16(rd)) as u64),
        Marker::U32 => Ok(try!(read_data_u32(rd)) as u64),
        Marker::U64 => Ok(try!(read_data_u64(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read up to 9 bytes from the reader (1 for marker and up to 8 for data) and interpret
/// them as a big-endian i64.
///
/// The function tries to decode only signed integer values that can potentially be negative.
#[unstable(reason = "not sure about name")]
pub fn read_i64_loosely<R>(rd: &mut R) -> Result<i64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::NegativeFixnum(val) => Ok(val as i64),
        Marker::I8  => Ok(try!(read_data_i8(rd))  as i64),
        Marker::I16 => Ok(try!(read_data_i16(rd)) as i64),
        Marker::I32 => Ok(try!(read_data_i32(rd)) as i64),
        Marker::I64 => Ok(try!(read_data_i64(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Yes, it is slower, because of ADT, but more convenient.
#[unstable(reason = "move to high-level module; complete; test")]
pub fn read_integer<R>(rd: &mut R) -> Result<Integer>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::NegativeFixnum(val) => Ok(Integer::I64(val as i64)),
        Marker::I8  => Ok(Integer::I64(try!(read_data_i8(rd))  as i64)),
        Marker::I16 => Ok(Integer::I64(try!(read_data_i16(rd)) as i64)),
        Marker::I32 => Ok(Integer::I64(try!(read_data_i32(rd)) as i64)),
        Marker::I64 => Ok(Integer::I64(try!(read_data_i64(rd)))),
        Marker::U64 => Ok(Integer::U64(try!(read_data_u64(rd)))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read a string's size from the reader.
///
/// String format family stores an byte array in 1, 2, 3, or 5 bytes of extra bytes in addition to
/// the size of the byte array.
pub fn read_str_len<R>(rd: &mut R) -> Result<u32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixedString(size) => Ok(size as u32),
        Marker::Str8  => Ok(try!(read_data_u8(rd))  as u32),
        Marker::Str16 => Ok(try!(read_data_u16(rd)) as u32),
        Marker::Str32 => Ok(try!(read_data_u32(rd))),
        _             => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

/// Tries to read a string data from the reader and copy it to the buffer provided.
///
/// On success returns a borrowed string type, allowing to view the copyed bytes as properly utf-8
/// string.
/// According to the spec, the string's data must to be encoded using utf-8.
///
/// # Failure
///
/// Returns `Err` in the following cases:
///
///  - if any IO error (including unexpected EOF) occurs, while reading an `rd`.
///  - if the `out` buffer size is too small to keep all copyed data.
///  - if the data is not utf-8, with a description as to why the provided data is not utf-8 and
///    with a size of bytes actually copyed to be able to get them from `out`.
///
/// # Examples
/// ```
/// use msgpack::core::decode::read_str;
///
/// let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
/// let mut out = [0u8; 16];
///
/// assert_eq!("le message", read_str(&mut &buf[..], &mut &mut out[..]).unwrap());
/// ```
#[unstable(reason = "less `as`")]
pub fn read_str<'r, R>(rd: &mut R, mut buf: &'r mut [u8]) -> result::Result<&'r str, DecodeStringError<'r>>
    where R: Read
{
    let len = try!(read_str_len(rd));
    let ulen = len as usize;

    if buf.len() < ulen {
        return Err(DecodeStringError::Core(Error::BufferSizeTooSmall(len)))
    }

    read_str_data(rd, len, &mut buf[0..ulen])
}

fn read_str_data<'r, R>(rd: &mut R, len: u32, buf: &'r mut[u8])
    -> result::Result<&'r str, DecodeStringError<'r>>
    where R: Read
{
    debug_assert_eq!(len as usize, buf.len());

    // We need cursor here, because in the common case we cannot guarantee, that copying will be
    // performed in a single step.
    let mut cur = Cursor::new(buf);

    // Trying to copy exact `len` bytes.
    match io::copy(&mut rd.take(len as u64), &mut cur) {
        Ok(size) if size == len as u64 => {
            // Release buffer owning from cursor.
            let buf = cur.into_inner();

            match from_utf8(buf) {
                Ok(decoded) => Ok(decoded),
                Err(err)    => Err(DecodeStringError::InvalidUtf8(buf, err)),
            }
        }
        Ok(size) => {
            let buf = cur.into_inner();
            Err(DecodeStringError::InvalidDataCopy(&buf[..size as usize], ReadError::UnexpectedEOF))
        }
        Err(err) => Err(DecodeStringError::Core(Error::InvalidDataRead(From::from(err)))),
    }
}

/// Tries to read a string data from the reader and make a borrowed slice from it.
#[unstable(reason = "it is better to return &str; may panic on len mismatch")]
pub fn read_str_ref(rd: &[u8]) -> Result<&[u8]> {
    let mut cur = io::Cursor::new(rd);
    let len = try!(read_str_len(&mut cur));
    let start = cur.position() as usize;
    Ok(&rd[start .. start + len as usize])
}

/// Tries to read up to 5 bytes from the reader and interpret them as a big-endian u32 array size.
///
/// Array format family stores a sequence of elements in 1, 3, or 5 bytes of extra bytes in
/// addition to the elements.
pub fn read_array_size<R>(rd: &mut R) -> Result<u32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixedArray(size) => Ok(size as u32),
        Marker::Array16 => {
            match rd.read_u16::<byteorder::BigEndian>() {
                Ok(size) => Ok(size as u32),
                Err(err) => Err(Error::InvalidDataRead(From::from(err))),
            }
        }
        Marker::Array32 => {
            match rd.read_u32::<byteorder::BigEndian>() {
                Ok(size) => Ok(size),
                Err(err) => Err(Error::InvalidDataRead(From::from(err))),
            }
        }
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

#[unstable = "documentation required"]
pub fn read_map_size<R>(rd: &mut R) -> Result<u32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixedMap(size) => Ok(size as u32),
        Marker::Map16 => Ok(try!(read_data_u16(rd)) as u32),
        Marker::Map32 => Ok(try!(read_data_u32(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

#[unstable = "documentation"]
pub fn read_f32<R>(rd: &mut R) -> Result<f32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::F32 => Ok(try!(read_data_f32(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

#[unstable = "docs"]
pub fn read_f64<R>(rd: &mut R) -> Result<f64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::F64 => Ok(try!(read_data_f64(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

pub fn read_bin_len<R>(rd: &mut R) -> Result<u32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::Bin8  => Ok(try!(read_data_u8(rd)) as u32),
        Marker::Bin16 => Ok(try!(read_data_u16(rd)) as u32),
        Marker::Bin32 => Ok(try!(read_data_u32(rd))),
        _             => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

#[unstable(reason = "docs; not sure about naming")]
pub fn read_bin_borrow(rd: &[u8]) -> Result<&[u8]> {
    let mut cur = io::Cursor::new(rd);
    let len = try!(read_bin_len(&mut cur)) as usize;

    let pos = cur.position() as usize;

    if rd.len() < pos + len {
        Err(Error::InvalidDataRead(ReadError::UnexpectedEOF))
    } else {
        Ok(&rd[pos .. pos + len])
    }
}

#[unstable = "docs"]
pub fn read_fixext1<R>(rd: &mut R) -> Result<(i8, u8)>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt1 => {
            let id   = try!(read_data_i8(rd));
            let data = try!(read_data_u8(rd));
            Ok((id, data))
        }
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

#[unstable = "docs"]
pub fn read_fixext2<R>(rd: &mut R) -> Result<(i8, u16)>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt2 => {
            let id   = try!(read_data_i8(rd));
            let data = try!(read_data_u16(rd));
            Ok((id, data))
        }
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

#[unstable = "docs; contains unsafe code"]
pub fn read_fixext4<R>(rd: &mut R) -> Result<(i8, [u8; 4])>
    where R: Read
{
    use std::mem;

    match try!(read_marker(rd)) {
        Marker::FixExt4 => {
            let id = try!(read_data_i8(rd));
            match rd.read_u32::<byteorder::LittleEndian>() {
                Ok(data) => {
                    let out : [u8; 4] = unsafe { mem::transmute(data) };
                    Ok((id, out))
                }
                Err(err) => Err(Error::InvalidDataRead(From::from(err))),
            }
        }
        _ => unimplemented!()
    }
}

#[unstable = "docs, error cases, type mismatch, unsufficient bytes, extra bytes"]
pub fn read_fixext8<R>(rd: &mut R) -> Result<(i8, [u8; 8])>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt8 => {
            let id = try!(read_data_i8(rd));
            let mut out = [0u8; 8];

            match io::copy(&mut rd.take(8), &mut &mut out[..]) {
                Ok(8) => Ok((id, out)),
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

#[unstable = "docs, error cases, type mismatch, unsufficient bytes, extra bytes"]
pub fn read_fixext16<R>(rd: &mut R) -> Result<(i8, [u8; 16])>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::FixExt16 => {
            let id = try!(read_data_i8(rd));
            let mut out = [0u8; 16];

            match io::copy(&mut rd.take(16), &mut &mut out[..]) {
                Ok(16) => Ok((id, out)),
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
}

#[derive(Debug, PartialEq)]
pub struct ExtMeta {
    pub typeid: i8,
    pub size: u32,
}

#[unstable = "docs, errors"]
pub fn read_ext_meta<R>(rd: &mut R) -> Result<ExtMeta>
    where R: Read
{
    let size = match try!(read_marker(rd)) {
        Marker::FixExt1  => 1,
        Marker::FixExt2  => 2,
        Marker::FixExt4  => 4,
        Marker::FixExt8  => 8,
        Marker::FixExt16 => 16,
        Marker::Ext8     => try!(read_data_u8(rd))  as u32,
        Marker::Ext16    => try!(read_data_u16(rd)) as u32,
        Marker::Ext32    => try!(read_data_u32(rd)),
        _ => unimplemented!()
    };

    let typeid = try!(read_data_i8(rd));
    let meta = ExtMeta { typeid: typeid, size: size };

    Ok(meta)
}

/// TODO: Markdown.
/// Contains: owned value decoding, owned error; owned result.
pub mod value {

use std::convert;
use std::io::Read;
use std::result;
use std::str::Utf8Error;

use super::{read_marker, read_data_u8, read_data_i32, read_str_data};
use super::super::{Marker, Value, Integer, ReadError, DecodeStringError};
use super::super::super::core;

#[derive(Debug, PartialEq)]
pub enum Error {
    Core(core::Error),
    InvalidDataCopy(Vec<u8>, ReadError),
    /// The decoded data is not valid UTF-8, provides the original data and the corresponding error.
    InvalidUtf8(Vec<u8>, Utf8Error),
}

impl convert::From<core::Error> for Error {
    fn from(err: core::Error) -> Error {
        Error::Core(err)
    }
}

impl<'a> convert::From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError) -> Error {
        match err {
            DecodeStringError::Core(err) => Error::Core(err),
            DecodeStringError::InvalidDataCopy(buf, err) => Error::InvalidDataCopy(buf.to_vec(), err),
            DecodeStringError::InvalidUtf8(buf, err) => Error::InvalidUtf8(buf.to_vec(), err),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

#[unstable(reason = "docs; examples; incomplete")]
pub fn read_value<R>(rd: &mut R) -> Result<Value>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::Null => Ok(Value::Null),
        Marker::PositiveFixnum(v) => Ok(Value::Integer(Integer::U64(v as u64))),
        Marker::I32  => Ok(Value::Integer(Integer::I64(try!(read_data_i32(rd)) as i64))),
        // TODO: Other integers.
        // TODO: Floats.
        Marker::Str8 => {
            let len = try!(read_data_u8(rd)) as u64;

            let mut buf = Vec::with_capacity(len as usize);
            buf.resize(len as usize, 0);

            Ok(Value::String(try!(read_str_data(rd, len as u32, &mut buf[..])).to_string()))
        }
        // TODO: Other strings.
        Marker::FixedArray(len) => {
            let mut vec = Vec::with_capacity(len as usize);

            for _ in 0..len {
                vec.push(try!(read_value(rd)));
            }

            Ok(Value::Array(vec))
        }
        // TODO: Map/Bin/Ext.
        _ => unimplemented!()
    }
}

} // mod value
