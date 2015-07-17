//! This module is UNSTABLE, the reason is - recently added.

use std::convert::From;
use std::io::{self, Read, BufRead};
use std::str::{from_utf8, Utf8Error};

use super::{read_marker};
use super::{
    ReadError,
    MarkerReadError,
};
use super::{BigEndianRead};

use super::super::init::Marker;
use super::super::value::ValueRef;

trait USizeCast {
    fn from(v: Self) -> Option<usize> where Self: Sized;
}

impl USizeCast for u8 {
    fn from(v: u8) -> Option<usize> {
        // Impossible to panic, since u8 always fits in usize.
        Some(v as usize)
    }
}

impl USizeCast for u16 {
    fn from(v: u16) -> Option<usize> {
        // TODO: This can overflow on 8-bit systems.
        Some(v as usize)
    }
}

impl USizeCast for u32 {
    fn from(v: u32) -> Option<usize> {
        // TODO: This can overflow on 8- and 16-bit systems.
        Some(v as usize)
    }
}

// TODO: Display trait.
#[derive(Debug)]
pub enum Error<'r> {
    /// Failed to fill the internal reader buffer.
    ///
    /// RMP tries to obtain the buffer at the beginning of read operation using `read_buf` function.
    ///
    /// According to the Rust documentation, `fill_buf` function will return an I/O error if the
    /// underlying reader was read, but returned an error.
    InvalidBufferFill(io::Error),
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
}

impl<'r> From<MarkerReadError> for Error<'r> {
    fn from(err: MarkerReadError) -> Error<'r> {
        Error::InvalidMarkerRead(From::from(err))
    }
}

fn read_length<R, D>(rd: &mut R) -> Result<D, ReadError>
    where R: Read,
          D: BigEndianRead
{
    D::read(rd).map_err(From::from)
}

fn read_str(buf: &[u8], len: usize) -> Result<&str, Error> {
    let buf = try!(read_bin(buf, len));

    // Try to decode sliced buffer as UTF-8.
    let res = try!(from_utf8(buf).map_err(|err| Error::InvalidUtf8(buf, err)));

    Ok(res)
}

fn read_bin(buf: &[u8], len: usize) -> Result<&[u8], Error> {
    if len > buf.len() {
        return Err(Error::InvalidDataRead(ReadError::UnexpectedEOF));
    }

    // Take a slice.
    let buf = &buf[..len];

    Ok(buf)
}

#[inline]
fn read_str_value<U>(buf: &[u8], len: U) -> Result<ValueRef, Error>
    where U: USizeCast
{
    let len = try!(U::from(len).ok_or(Error::InvalidLengthSize));
    let res = try!(read_str(buf, len));

    Ok(ValueRef::String(res))
}

#[inline]
fn read_bin_value<U>(buf: &[u8], len: U) -> Result<ValueRef, Error>
    where U: USizeCast
{
    let len = try!(U::from(len).ok_or(Error::InvalidLengthSize));
    let res = try!(read_bin(buf, len));

    Ok(ValueRef::Binary(res))
}

// NOTE: Consumes nothing from the given `BufRead` both on success and fail.
pub fn read_value_ref<R>(rd: &mut R) -> Result<ValueRef, Error>
    where R: BufRead
{
    let mut buf = try!(rd.fill_buf().map_err(|err| Error::InvalidBufferFill(err)));

    // Reading the marker involves either 1 byte read or nothing. On success consumes strictly
    // 1 byte from the `buf`, not from the `rd`.
    let marker = try!(read_marker(&mut buf));

    let val = match marker {
        Marker::FixedString(len) => {
            try!(read_str_value(buf, len))
        }
        Marker::Str8 => {
            let len: u8 = try!(read_length(&mut buf).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_str_value(buf, len))
        }
        Marker::Str16 => {
            let len: u16 = try!(read_length(&mut buf).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_str_value(buf, len))
        }
        Marker::Str32 => {
            let len: u32 = try!(read_length(&mut buf).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_str_value(buf, len))
        }
        Marker::Bin8 => {
            let len: u8 = try!(read_length(&mut buf).map_err(|err| Error::InvalidLengthRead(err)));
            try!(read_bin_value(buf, len))
        }
        Marker::Bin16 => {
            unimplemented!();
        }
        Marker::Bin32 => {
            unimplemented!();
        }
        _ => unimplemented!(),
    };

    Ok(val)
}

// TODO: Concepts: less code; more tests; fast refactoring after each test; no more than 5 min for
// each red-yellow-green.
