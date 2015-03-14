#![feature(core)]
#![feature(io)]

extern crate byteorder;

use std::num::FromPrimitive;
use std::error;
use std::io;
use std::io::Read;

use byteorder::{ReadBytesExt};

pub const MSGPACK_VERSION : u32 = 5;

const FIXSTR_SIZE : u8 = 0x1f;

enum Marker {
    Fixnum(u8),
    Null,
    True,
    False,
    U8,
    U16,
    U32,
    U64,
    FixedString(u8),
    Str8,
}

impl FromPrimitive for Marker {
    fn from_i64(n: i64) -> Option<Marker> {
        FromPrimitive::from_u64(n as u64)
    }

    fn from_u64(n: u64) -> Option<Marker> {
        match n {
            val @ 0x00 ... 0x7f => Some(Marker::Fixnum(val as u8)),
            val @ 0xa0 ... 0xbf => Some(Marker::FixedString((val as u8) & FIXSTR_SIZE)),
            0xc0 => Some(Marker::Null),
            0xc2 => Some(Marker::False),
            0xc3 => Some(Marker::True),
            0xcc => Some(Marker::U8),
            0xcd => Some(Marker::U16),
            0xce => Some(Marker::U32),
            0xcf => Some(Marker::U64),
            0xd9 => Some(Marker::Str8),
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
pub fn read_positive_fixnum_exact<R>(rd: &mut R) -> Result<u8>
    where R: Read
{
    match read_marker(rd) {
        Ok(Marker::Fixnum(val)) => Ok(val),
        Ok(..) => Err(Error::InvalidMarker(MarkerError::TypeMismatch)),
        Err(err) => Err(err),
    }
}

/// Tries to read and decode an unsigned integer from the reader.
pub fn read_u64<R>(rd: &mut R) -> Result<u64>
    where R: Read
{
    match read_marker(rd) {
        Ok(Marker::Fixnum(val)) => Ok(val as u64),
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
        _ => unimplemented!()
    }
}

/// Tries to read a string data from the reader and copy it to the buffer provided.
///
/// According to the spec, the string's data must to be encoded using UTF-8.
/// Returns number of bytes actually read.
pub fn read_str_data<R>(rd: &mut R, buf: &mut [u8]) -> Result<u32>
    where R: Read
{
    unimplemented!();
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

    assert_eq!(0u8, read_positive_fixnum_exact(&mut cur).unwrap());
    assert_eq!(1, cur.position());

    assert_eq!(127u8, read_positive_fixnum_exact(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(32u8, read_positive_fixnum_exact(&mut cur).unwrap());
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
    let buf: &[u8] = &[0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarker(MarkerError::Unexpected(0x80)), read_u64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixstr_read_str_len() {
    let buf: &[u8] = &[0xaa];
    let mut cur = Cursor::new(buf);

    assert_eq!(10, read_str_len(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str8_read_str_len() {
    let buf: &[u8] = &[0xd9, 0x0a];
    let mut cur = Cursor::new(buf);

    assert_eq!(10, read_str_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}


//#[test]
//fn from_str_fixstr() {
//    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
//    let out: &[u8] = &[0u8; 16];
//    let mut cur = Cursor::new(buf);

//    assert_eq!(10, read_str_len(&mut cur).unwrap());
//    assert_eq!(1, cur.position());
//}

} // mod testing
