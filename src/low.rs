use std; // For std::result, use std::result instead.
use std::num::FromPrimitive;
use std::error;
use std::io;
use std::io::Read;

use byteorder::{self, ReadBytesExt};

pub const MSGPACK_VERSION : u32 = 5;

const FIXSTR_SIZE   : u8 = 0x1f;
const FIXARRAY_SIZE : u8 = 0x0f;
const FIXMAP_SIZE   : u8 = 0x0f;

enum Marker {
    PositiveFixnum(u8),
    NegativeFixnum(i8),
    Null,
    True,
    False,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    FixedString(u8),
    Str8,
    Str16,
    Str32,
    Bin8,
    Bin16,
    Bin32,
    FixedArray(u8),
    Array16,
    Array32,
    FixedMap(u8),
    Map16,
    Map32,
    FixExt1,
    FixExt2,
    FixExt4,
    FixExt8,
    FixExt16,
    Ext8,
    Ext16,
    Ext32,
}

impl FromPrimitive for Marker {
    fn from_i64(n: i64) -> Option<Marker> {
        FromPrimitive::from_u64(n as u64)
    }

    fn from_u64(n: u64) -> Option<Marker> {
        match n {
            val @ 0x00 ... 0x7f => Some(Marker::PositiveFixnum(val as u8)),
            val @ 0xe0 ... 0xff => Some(Marker::NegativeFixnum(val as i8)),
            val @ 0x80 ... 0x8f => Some(Marker::FixedMap((val as u8) & FIXMAP_SIZE)),
            val @ 0x90 ... 0x9f => Some(Marker::FixedArray((val as u8) & FIXARRAY_SIZE)),
            val @ 0xa0 ... 0xbf => Some(Marker::FixedString((val as u8) & FIXSTR_SIZE)),
            0xc0 => Some(Marker::Null),
            0xc1 => None, // Marked in MessagePack spec as never used.
            0xc2 => Some(Marker::False),
            0xc3 => Some(Marker::True),
            0xc4 => Some(Marker::Bin8),
            0xc5 => Some(Marker::Bin16),
            0xc6 => Some(Marker::Bin32),
            0xc7 => Some(Marker::Ext8),
            0xc8 => Some(Marker::Ext16),
            0xc9 => Some(Marker::Ext32),
            0xca => Some(Marker::F32),
            0xcb => Some(Marker::F64),
            0xcc => Some(Marker::U8),
            0xcd => Some(Marker::U16),
            0xce => Some(Marker::U32),
            0xcf => Some(Marker::U64),
            0xd0 => Some(Marker::I8),
            0xd1 => Some(Marker::I16),
            0xd2 => Some(Marker::I32),
            0xd3 => Some(Marker::I64),
            0xd4 => Some(Marker::FixExt1),
            0xd5 => Some(Marker::FixExt2),
            0xd6 => Some(Marker::FixExt4),
            0xd7 => Some(Marker::FixExt8),
            0xd8 => Some(Marker::FixExt16),
            0xd9 => Some(Marker::Str8),
            0xda => Some(Marker::Str16),
            0xdb => Some(Marker::Str32),
            0xdc => Some(Marker::Array16),
            0xdd => Some(Marker::Array32),
            0xde => Some(Marker::Map16),
            0xdf => Some(Marker::Map32),
            _ => None,
        }
    }
}

/// An error type for reading bytes from the reader.
///
/// This is a thin wrapper over the standard `io::Error` type. Namely, it adds one additional error
/// case: an unexpected EOF.
#[derive(PartialEq, Debug)]
pub enum ReadError {
    UnexpectedEOF,
    IO(io::Error),
}

impl error::FromError<io::Error> for ReadError {
    fn from_error(err: io::Error) -> ReadError { ReadError::IO(err) }
}

impl error::FromError<ReadError> for io::Error {
    fn from_error(err: ReadError) -> io::Error {
        match err {
            ReadError::IO(err) => err,
            ReadError::UnexpectedEOF => io::Error::new(io::ErrorKind::Other, "unexpected EOF", None)
        }
    }
}

impl error::FromError<byteorder::Error> for ReadError {
    fn from_error(err: byteorder::Error) -> ReadError {
        match err {
            byteorder::Error::UnexpectedEOF => ReadError::UnexpectedEOF,
            byteorder::Error::Io(err) => ReadError::IO(err),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum MarkerError {
    TypeMismatch,
    Unexpected(u8),
}

#[derive(PartialEq, Debug)]
pub enum Error {
    InvalidMarker(MarkerError),     // Marker type error.
    InvalidMarkerRead(ReadError),   // IO error while reading marker.
    InvalidDataRead(ReadError),     // IO error while reading data.
    BufferSizeTooSmall(u32),        // Too small buffer provided to copy all the data.
}

pub type Result<T> = std::result::Result<T, Error>;

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
        Err(err) => Err(Error::InvalidMarkerRead(error::FromError::from_error(err))),
    }
}

/// Tries to read nil value from the reader.
pub fn read_nil<R>(rd: &mut R) -> Result<()>
    where R: Read
{
    let marker = try!(read_marker(rd));

    match marker {
        Marker::Null => Ok(()),
        _            => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

/// Tries to read bool value from the reader.
pub fn read_bool<R>(rd: &mut R) -> Result<bool>
    where R: Read
{
    let marker = try!(read_marker(rd));

    match marker {
        Marker::True  => Ok(true),
        Marker::False => Ok(false),
        _             => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

// Tries to read exact positive fixnum from the reader.
pub fn read_pfix<R>(rd: &mut R) -> Result<u8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::PositiveFixnum(val) => Ok(val),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

pub fn read_nfix<R>(rd: &mut R) -> Result<i8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::NegativeFixnum(val) => Ok(val),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i8 value from the reader.
pub fn read_i8<R>(rd: &mut R) -> Result<i8>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I8 => Ok(try!(read_i8_data(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i16 value from the reader.
pub fn read_i16<R>(rd: &mut R) -> Result<i16>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I16 => Ok(try!(read_i16_data(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i32 value from the reader.
pub fn read_i32<R>(rd: &mut R) -> Result<i32>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I32 => Ok(try!(read_i32_data(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read strictly i64 value from the reader.
pub fn read_i64<R>(rd: &mut R) -> Result<i64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::I64 => Ok(try!(read_i64_data(rd))),
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
    }
}

/// Tries to read and decode an unsigned integer from the reader.
pub fn read_u64<R>(rd: &mut R) -> Result<u64>
    where R: Read
{
    match read_marker(rd) {
        Ok(Marker::PositiveFixnum(val)) => Ok(val as u64),
        Ok(Marker::U8) => {
            match rd.read_u8() {
                Ok(val)  => Ok(val as u64),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        Ok(Marker::U16) => {
            match rd.read_u16::<byteorder::BigEndian>() {
                Ok(val)  => Ok(val as u64),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        Ok(Marker::U32) => {
            match rd.read_u32::<byteorder::BigEndian>() {
                Ok(val)  => Ok(val as u64),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        Ok(Marker::U64) => {
            match rd.read_u64::<byteorder::BigEndian>() {
                Ok(val)  => Ok(val),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        Ok(..)   => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
        Err(err) => Err(err),
    }
}

/// Tries to read exactly 1 byte from the reader and interpret it as an i8.
fn read_i8_data<R>(rd: &mut R) -> Result<i8>
    where R: Read
{
    match rd.read_i8() {
        Ok(val)  => Ok(val),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

/// Tries to read exactly 2 bytes from the reader and interpret them as a big-endian i16.
fn read_i16_data<R>(rd: &mut R) -> Result<i16>
    where R: Read
{
    match rd.read_i16::<byteorder::BigEndian>() {
        Ok(val)  => Ok(val),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

/// Tries to read exactly 4 bytes from the reader and interpret them as a big-endian i32.
fn read_i32_data<R>(rd: &mut R) -> Result<i32>
    where R: Read
{
    match rd.read_i32::<byteorder::BigEndian>() {
        Ok(val)  => Ok(val),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

/// Tries to read exactly 8 bytes from the reader and interpret them as a big-endian i64.
fn read_i64_data<R>(rd: &mut R) -> Result<i64>
    where R: Read
{
    match rd.read_i64::<byteorder::BigEndian>() {
        Ok(val)  => Ok(val),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

pub enum Integer {
    U64(u64),
    I64(i64),
}

pub enum Float {
    F32(f32),
    F64(f64),
}

pub enum Value {
    Null,
    Integer(Integer),
    Float(Float),
}

/// Tries to read up to 9 bytes from the reader (1 for marker and up to 8 for data) and interpret
/// them as a big-endian i64.
/// TODO: Deserialization: nfix, pfix, int 8/16/32/64 and uint 8/16/32/64 -> Integer (i64|u64).
pub fn read_integer<R>(rd: &mut R) -> Result<i64>
    where R: Read
{
    match try!(read_marker(rd)) {
        Marker::NegativeFixnum(val) => Ok(val as i64),
        Marker::I8  => Ok(try!(read_i8_data(rd))  as i64),
        Marker::I16 => Ok(try!(read_i16_data(rd)) as i64),
        Marker::I32 => Ok(try!(read_i32_data(rd)) as i64),
        Marker::I64 => Ok(try!(read_i64_data(rd))),
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
    let marker = try!(read_marker(rd));

    match marker {
        Marker::FixedString(size) => Ok(size as u32),
        Marker::Str8 => {
            match rd.read_u8() {
                Ok(size) => Ok(size as u32),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        Marker::Str16 => {
            match rd.read_u16::<byteorder::BigEndian>() {
                Ok(size) => Ok(size as u32),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        Marker::Str32 => {
            match rd.read_u32::<byteorder::BigEndian>() {
                Ok(size) => Ok(size),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

/// Tries to read a string data from the reader and copy it to the buffer provided.
///
/// According to the spec, the string's data must to be encoded using UTF-8.
/// Returns number of bytes actually read.
pub fn read_str<R>(rd: &mut R, mut buf: &mut [u8]) -> Result<u32>
    where R: Read
{
    let len = try!(read_str_len(rd));
    if buf.len() < len as usize {
        return Err(Error::BufferSizeTooSmall(len))
    }

    // TODO: WTF? Copy only `len` bytes.
    match io::copy(rd, &mut buf) {
        Ok(size) => Ok(size as u32),
        Err(..) => unimplemented!(),
    }
}

/// Tries to read a string data from the reader and make a borrowed slice from it.
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
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        Marker::Array32 => {
            match rd.read_u32::<byteorder::BigEndian>() {
                Ok(size) => Ok(size),
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
            }
        }
        _ => Err(Error::InvalidMarker(MarkerError::TypeMismatch))
    }
}

fn read_data_i8<R>(rd: &mut R) -> Result<i8>
    where R: Read
{
    match rd.read_i8() {
        Ok(data) => Ok(data),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

fn read_data_u8<R>(rd: &mut R) -> Result<u8>
    where R: Read
{
    match rd.read_u8() {
        Ok(data) => Ok(data),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

fn read_data_u16<R>(rd: &mut R) -> Result<u16>
    where R: Read
{
    match rd.read_u16::<byteorder::BigEndian>() {
        Ok(data) => Ok(data),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

fn read_data_u32<R>(rd: &mut R) -> Result<u32>
    where R: Read
{
    match rd.read_u32::<byteorder::BigEndian>() {
        Ok(data) => Ok(data),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

fn read_data_f32<R>(rd: &mut R) -> Result<f32>
    where R: Read
{
    match rd.read_f32::<byteorder::BigEndian>() {
        Ok(data) => Ok(data),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
    }
}

fn read_data_f64<R>(rd: &mut R) -> Result<f64>
    where R: Read
{
    match rd.read_f64::<byteorder::BigEndian>() {
        Ok(data) => Ok(data),
        Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
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
                Err(err) => Err(Error::InvalidDataRead(error::FromError::from_error(err))),
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
    typeid: i8,
    size: u32,
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

#[cfg(test)]
mod testing {

use std::io::{Cursor};

use super::*;

#[test]
fn from_nil() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!((), read_nil(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nil_invalid_marker() {
    let buf: &[u8] = &[0xc1];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::Unexpected(0xc1)), read_nil(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nil_invalid_marker_read() {
    let buf: &[u8] = &[];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarkerRead(ReadError::UnexpectedEOF),
        read_nil(&mut cur).err().unwrap());
    assert_eq!(0, cur.position());
}

#[test]
fn from_bool_false() {
    let buf: &[u8] = &[0xc2];
    let mut cur = Cursor::new(buf);

    assert_eq!(false, read_bool(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_bool_true() {
    let buf: &[u8] = &[0xc3];
    let mut cur = Cursor::new(buf);

    assert_eq!(true, read_bool(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_positive_fixnum() {
    let buf: &[u8] = &[0x00, 0x7f, 0x20];
    let mut cur = Cursor::new(buf);

    assert_eq!(0u8, read_pfix(&mut cur).unwrap());
    assert_eq!(1, cur.position());

    assert_eq!(127u8, read_pfix(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(32u8, read_pfix(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_unsigned_fixnum() {
    let buf: &[u8] = &[0x00, 0x7f, 0x20];
    let mut cur = Cursor::new(buf);

    assert_eq!(0u64, read_u64(&mut cur).unwrap());
    assert_eq!(1, cur.position());

    assert_eq!(127u64, read_u64(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(32u64, read_u64(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_unsigned_u8() {
    let buf: &[u8] = &[0xcc, 0x80, 0xcc, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(128u64, read_u64(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(255u64, read_u64(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_unsigned_u8_invalid_data_read() {
    let buf: &[u8] = &[0xcc];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_u64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u16() {
    let buf: &[u8] = &[0xcd, 0x01, 0x00, 0xcd, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(256u64, read_u64(&mut cur).unwrap());
    assert_eq!(3, cur.position());

    assert_eq!(65535u64, read_u64(&mut cur).unwrap());
    assert_eq!(6, cur.position());
}

#[test]
fn from_unsigned_u16_invalid_data_read() {
    let buf: &[u8] = &[0xcd];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_u64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u32() {
    let buf: &[u8] = &[0xce, 0x00, 0x01, 0x00, 0x00, 0xce, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65536u64, read_u64(&mut cur).unwrap());
    assert_eq!(5, cur.position());

    assert_eq!(4294967295u64, read_u64(&mut cur).unwrap());
    assert_eq!(10, cur.position());
}

#[test]
fn from_unsigned_u32_invalid_data_read() {
    let buf: &[u8] = &[0xce];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_u64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u64() {
    let buf: &[u8] = &[
        0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
    ];
    let mut cur = Cursor::new(buf);

    assert_eq!(4294967296u64, read_u64(&mut cur).unwrap());
    assert_eq!(9, cur.position());

    assert_eq!(18446744073709551615u64, read_u64(&mut cur).unwrap());
    assert_eq!(18, cur.position());
}

#[test]
fn from_unsigned_u64_invalid_data_read() {
    let buf: &[u8] = &[0xcf];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_u64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_invalid_marker() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_u64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_invalid_unknown_marker() {
    let buf: &[u8] = &[0xc1];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::Unexpected(0xc1)), read_u64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixstr_min_read_str_len() {
    let buf: &[u8] = &[0xa0];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixstr_rnd_read_str_len() {
    let buf: &[u8] = &[0xaa];
    let mut cur = Cursor::new(buf);

    assert_eq!(10, read_str_len(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixstr_max_read_str_len() {
    let buf: &[u8] = &[0xbf];
    let mut cur = Cursor::new(buf);

    assert_eq!(31, read_str_len(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str8_min_read_str_len() {
    let buf: &[u8] = &[0xd9, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str8_rnd_read_str_len() {
    let buf: &[u8] = &[0xd9, 0x0a];
    let mut cur = Cursor::new(buf);

    assert_eq!(10, read_str_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str8_read_str_len_eof() {
    let buf: &[u8] = &[0xd9];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str8_max_read_str_len() {
    let buf: &[u8] = &[0xd9, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(255, read_str_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str16_min_read_str_len() {
    let buf: &[u8] = &[0xda, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_str16_max_read_str_len() {
    let buf: &[u8] = &[0xda, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65535, read_str_len(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_str16_read_str_len_eof() {
    let buf: &[u8] = &[0xda, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str32_min_read_str_len() {
    let buf: &[u8] = &[0xdb, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_str32_max_read_str_len() {
    let buf: &[u8] = &[0xdb, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(4294967295, read_str_len(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_str32_read_str_len_eof() {
    let buf: &[u8] = &[0xdb, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_null_read_str_len() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str_strfix() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 16];

    assert_eq!(10, read_str(&mut cur, &mut out).unwrap());
    assert_eq!(11, cur.position());

    assert!(buf[1..11] == out[0..10]);
}

#[test]
fn from_str_strfix_buffer_too_small() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 9];

    assert_eq!(Error::BufferSizeTooSmall(10), read_str(&mut cur, &mut out).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str_strfix_ref() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];

    let out = read_str_ref(&buf).unwrap();

    assert_eq!(10, out.len());
    assert!(buf[1..11] == out[0..10])
}

#[test]
fn from_nfix_min() {
    let buf: &[u8] = &[0xe0];
    let mut cur = Cursor::new(buf);

    assert_eq!(-32, read_nfix(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nfix_max() {
    let buf: &[u8] = &[0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(-1, read_nfix(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nfix_type_mismatch() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_nfix(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i8_min() {
    let buf: &[u8] = &[0xd0, 0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(-128, read_i8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_max() {
    let buf: &[u8] = &[0xd0, 0x7f];
    let mut cur = Cursor::new(buf);

    assert_eq!(127, read_i8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_i8(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i8_unexpected_eof() {
    let buf: &[u8] = &[0xd0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_i8(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i16_min() {
    let buf: &[u8] = &[0xd1, 0x80, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-32768, read_i16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_max() {
    let buf: &[u8] = &[0xd1, 0x7f, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(32767, read_i16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_i16(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i16_unexpected_eof() {
    let buf: &[u8] = &[0xd1, 0x7f];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_i16(&mut cur).err().unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i32_min() {
    let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-2147483648, read_i32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_max() {
    let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(2147483647, read_i32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_i32(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i32_unexpected_eof() {
    let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_i32(&mut cur).err().unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_i64_min() {
    let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-9223372036854775808, read_i64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_max() {
    let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(9223372036854775807, read_i64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_i64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i64_unexpected_eof() {
    let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_i64(&mut cur).err().unwrap());
    assert_eq!(8, cur.position());
}

#[test]
fn from_nfix_min_read_integer() {
    let buf: &[u8] = &[0xe0];
    let mut cur = Cursor::new(buf);

    assert_eq!(-32, read_integer(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nfix_max_read_integer() {
    let buf: &[u8] = &[0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(-1, read_integer(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i8_min_read_integer() {
    let buf: &[u8] = &[0xd0, 0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(-128, read_integer(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_max_read_integer() {
    let buf: &[u8] = &[0xd0, 0x7f];
    let mut cur = Cursor::new(buf);

    assert_eq!(127, read_integer(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i16_min_read_integer() {
    let buf: &[u8] = &[0xd1, 0x80, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-32768, read_integer(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_max_read_integer() {
    let buf: &[u8] = &[0xd1, 0x7f, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(32767, read_integer(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i32_min_read_integer() {
    let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-2147483648, read_integer(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_max_read_integer() {
    let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(2147483647, read_integer(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i64_min_read_integer() {
    let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-9223372036854775808, read_integer(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_max_read_integer() {
    let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(9223372036854775807, read_integer(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_empty_array_read_size() {
    let buf: &[u8] = &[0x90];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_array_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixarray_max_read_size() {
    let buf: &[u8] = &[
        0x9f,
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e
    ];
    let mut cur = Cursor::new(buf);

    assert_eq!(15, read_array_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_array16_min_read_size() {
    let buf: &[u8] = &[0xdc, 0x00, 0x10];
    let mut cur = Cursor::new(buf);

    assert_eq!(16, read_array_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_array16_max_read_size() {
    let buf: &[u8] = &[0xdc, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65535, read_array_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_array16_unexpected_eof_read_size() {
    let buf: &[u8] = &[0xdc, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_array_size(&mut cur).err().unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_array32_min_read_size() {
    let buf: &[u8] = &[0xdd, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_array_size(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_array32_max_read_size() {
    let buf: &[u8] = &[0xdd, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(4294967295, read_array_size(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_array32_unexpected_eof_read_size() {
    let buf: &[u8] = &[0xdd, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF), read_array_size(&mut cur).err().unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_null_read_array_size() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_array_size(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixmap_min_read_size() {
    let buf: &[u8] = &[0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_map_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixmap_max_read_size() {
    let buf: &[u8] = &[0x8f];
    let mut cur = Cursor::new(buf);

    assert_eq!(15, read_map_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_map16_min_read_size() {
    let buf: &[u8] = &[0xde, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_map_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_map16_max_read_size() {
    let buf: &[u8] = &[0xde, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65535, read_map_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_map32_min_read_size() {
    let buf: &[u8] = &[0xdf, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_map_size(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_null_read_map_size() {
    let buf: &[u8] = &[0xc0, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_map_size(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_f32_zero_plus() {
    let buf: &[u8] = &[0xca, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0.0, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_f32_max() {
    let buf: &[u8] = &[0xca, 0x7f, 0x7f, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(3.4028234e38_f32, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_f32_inf() {
    use std::f32;

    let buf: &[u8] = &[0xca, 0x7f, 0x80, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f32::INFINITY, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_f32_neg_inf() {
    use std::f32;

    let buf: &[u8] = &[0xca, 0xff, 0x80, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f32::NEG_INFINITY, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_null_read_f32() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_f32(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_f64_zero_plus() {
    let buf: &[u8] = &[0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0.0, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_f64_zero_minus() {
    let buf: &[u8] = &[0xcb, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-0.0, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_f64_inf() {
    use std::f64;

    let buf: &[u8] = &[0xcb, 0x7f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f64::INFINITY, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_f64_neg_inf() {
    use std::f64;

    let buf: &[u8] = &[0xcb, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f64::NEG_INFINITY, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_null_read_f64() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::TypeMismatch), read_f64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixext1_read_fixext1() {
    let buf: &[u8] = &[0xd4, 0x01, 0x02];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, 2), read_fixext1(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_fixext2_read_fixext2() {
    let buf: &[u8] = &[0xd5, 0x01, 0x00, 0x02];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, 2), read_fixext2(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_fixext4_read_fixext4() {
    let buf: &[u8] = &[0xd6, 0x01, 0x00, 0x00, 0x00, 0x02];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, [0x00, 0x00, 0x00, 0x02]), read_fixext4(&mut cur).unwrap());
    assert_eq!(6, cur.position());
}

#[test]
fn from_fixext8_read_fixext8() {
    let buf: &[u8] = &[0xd7, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
               read_fixext8(&mut cur).unwrap());
    assert_eq!(10, cur.position());
}

#[test]
fn from_fixext16_read_fixext16() {
    let buf: &[u8] = &[0xd8, 0x01,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
               read_fixext16(&mut cur).unwrap());
    assert_eq!(18, cur.position());
}

#[test]
fn from_fixext1_read_ext_meta() {
    let buf: &[u8] = &[0xd4, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 1 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext2_read_ext_meta() {
    let buf: &[u8] = &[0xd5, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 2 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext4_read_ext_meta() {
    let buf: &[u8] = &[0xd6, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 4 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext8_read_ext_meta() {
    let buf: &[u8] = &[0xd7, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 8 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext16_read_ext_meta() {
    let buf: &[u8] = &[0xd8, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 16 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_ext8_read_ext_meta() {
    let buf: &[u8] = &[0xc7, 0xff, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 255 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_ext16_read_ext_meta() {
    let buf: &[u8] = &[0xc8, 0xff, 0xff, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 65535 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_ext32_read_ext_meta() {
    let buf: &[u8] = &[0xc9, 0xff, 0xff, 0xff, 0xff, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 4294967295 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(6, cur.position());
}

} // mod testing
