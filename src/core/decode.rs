use std::convert::From;
use std::io;
use std::io::Read;
use std::num::FromPrimitive;
use std::str::from_utf8;

use byteorder::{self, ReadBytesExt};

use super::{Marker, Error, MarkerError, ReadError, Result};

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
pub fn read_i8<R>(rd: &mut R) -> Result<i8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I8 => Ok(try!(read_data_i8(rd))),
        _          => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i16 value from the reader.
pub fn read_i16<R>(rd: &mut R) -> Result<i16>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I16 => Ok(try!(read_data_i16(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i32 value from the reader.
pub fn read_i32<R>(rd: &mut R) -> Result<i32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I32 => Ok(try!(read_data_i32(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i64 value from the reader.
pub fn read_i64<R>(rd: &mut R) -> Result<i64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I64 => Ok(try!(read_data_i64(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
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

#[unstable(reason = "docs")]
pub fn read_u16<R>(rd: &mut R) -> Result<u16>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::U16 => Ok(try!(read_data_u16(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

#[unstable(reason = "docs")]
pub fn read_u32<R>(rd: &mut R) -> Result<u32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::U32 => Ok(try!(read_data_u32(rd))),
        _           => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

#[unstable(reason = "docs")]
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

#[derive(Clone, Debug, PartialEq)]
pub enum Integer {
    U64(u64),
    I64(i64),
}

pub enum Float {
    F32(f32),
    F64(f64),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(Integer),
    String(String),
}

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
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
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
/// According to the spec, the string's data must to be encoded using UTF-8.
#[unstable(reason = "docs; example; signature; less `as`")]
pub fn read_str<'r, R>(rd: &mut R, mut buf: &'r mut [u8]) -> Result<&'r str>
    where R: Read
{
    let len = try!(read_str_len(rd));

    if buf.len() < len as usize {
        return Err(Error::BufferSizeTooSmall(len))
    }

    match io::copy(&mut rd.take(len as u64), &mut &mut buf[..len as usize]) {
        Ok(size) if size == len as u64 => {
            match from_utf8(&buf[..len as usize]) {
                Ok(decoded) => Ok(decoded),
                Err(err)    => Err(Error::InvalidUtf8(len, err)),
            }
        }
        Ok(size) => Err(Error::InvalidDataCopy(size as u32, ReadError::UnexpectedEOF)),
        Err(err) => Err(Error::InvalidDataRead(From::from(err))),
    }
}

/// Tries to read a string data from the reader and make a borrowed slice from it.
#[unstable(reason = "it is better to return &str")]
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

pub fn read_value<R>(rd: &mut R) -> Result<Value>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I32  => Ok(Value::Integer(Integer::I64(try!(read_data_i32(rd)) as i64))),
        Marker::Str8 => {
            let len = try!(read_data_u8(rd)) as u64;
            let mut buf = Vec::with_capacity(len as usize);
            match io::copy(&mut rd.take(len), &mut buf) {
                Ok(size) if size == len => {
                    Ok(Value::String(String::from_utf8(buf).unwrap())) // TODO: Do not unwrap, use Error.
                }
                Ok(..)  => unimplemented!(), // TODO: Return Error with read buffer anyway?
                Err(..) => unimplemented!(),
            }
        }
        _ => unimplemented!()
    }
}

#[cfg(test)]
mod testing {

extern crate test;

use super::*;
use self::test::Bencher;

#[bench]
fn from_i64_read_i64_loosely(b: &mut Bencher) {
    let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

    b.iter(|| {
        let res = read_i64_loosely(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}

#[bench]
fn from_i64_read_integer(b: &mut Bencher) {
    let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

    b.iter(|| {
        let res = read_integer(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}

#[bench]
fn from_i8_read_i8(b: &mut Bencher) {
    let buf = [0xd0, 0xff];

    b.iter(|| {
        let res = read_i8(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}

#[bench]
fn from_u8_read_u64_loosely(b: &mut Bencher) {
    let buf = [0xcc, 0xff];

    b.iter(|| {
        let res = read_u64_loosely(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}

} // mod testing
