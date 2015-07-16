//! This module is UNSTABLE, the reason is - just added.

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

    // length overflow

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

// NOTE: Consumes nothing from the given `BufRead` either on success or fail.
pub fn read_value_ref<R>(rd: &mut R) -> Result<ValueRef, Error>
    where R: BufRead
{
    let mut buf = try!(rd.fill_buf().map_err(|err| Error::InvalidBufferFill(err)));

    // Reading the marker involves either 1 byte read or nothing. On success consumes strictly
    // 1 byte from the `buf`, not from the `rd`.
    let marker = try!(read_marker(&mut buf));

    let val = match marker {
        Marker::FixedString(len) => {
            // Impossible to panic, since u8 always fits in usize.
            let len = len as usize;

            if len > buf.len() {
                return Err(Error::InvalidDataRead(ReadError::UnexpectedEOF));
            }

            // Take a slice.
            let buf = &buf[..len];

            // Try to decode sliced buffer as UTF-8.
            let res = try!(from_utf8(buf).map_err(|err| Error::InvalidUtf8(buf, err)));

            ValueRef::String(res)
        }
        Marker::Str8 => {
            let len: u8 = try!(read_length(&mut buf).map_err(|err| Error::InvalidLengthRead(err)));

            // Impossible to panic, since u8 always fits in usize.
            let len = len as usize;

            if len > buf.len() {
                return Err(Error::InvalidDataRead(ReadError::UnexpectedEOF));
            }

            // Take a slice.
            let buf = &buf[..len];

            // Try to decode sliced buffer as UTF-8.
            let res = try!(from_utf8(buf).map_err(|err| Error::InvalidUtf8(buf, err)));

            ValueRef::String(res)
        }
        _ => unimplemented!(),
    };

    Ok(val)
}

// TODO: Concepts: less code; more tests; fast refactoring after each test; no more than 5 min for
// each red-yellow-green.
