//! This module is UNSTABLE, the reason is - just added.

use std::convert::From;
use std::io::{self, Read, BufRead};
use std::str::from_utf8;

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
pub enum Error {
    /// Unable to fill the internal reader buffer.
    ///
    /// According to the Rust documentation, `fill_buf` function will return an I/O error if the
    /// underlying reader was read, but returned an error.
    InvalidBufferFill(io::Error),
    /// Failed to read the marker value.
    InvalidMarkerRead(ReadError),
    /// Failed to read string/array/map size.
    InvalidLengthRead(ReadError),
    // insuffifient bytes
    // invalid string length read (IO)
    // length overflow
    // invalid utf8
}

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        Error::InvalidMarkerRead(From::from(err))
    }
}

fn read_length<R, D>(rd: &mut R) -> Result<D, Error>
    where R: Read,
          D: BigEndianRead
{
    D::read(rd).map_err(|err| Error::InvalidLengthRead(From::from(err)))
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
        Marker::Str8 => {
            let len: u8 = try!(read_length(&mut buf));

            let len = len as usize; // TODO: May panic.
            // TODO: Check buffer length.
            let res = from_utf8(&buf[..len]).unwrap(); // TODO: May fail (not UTF-8), return &[u8] otherwise.
            ValueRef::String(res)
        }
        _ => unimplemented!(),
    };

    Ok(val)
}

// TODO: Concepts: less code; more tests; fast refactoring after each test; no more than 5 min for
// each red-yellow-green.
