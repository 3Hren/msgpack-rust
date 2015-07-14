use std::convert::From;
use std::io::BufRead;
use std::str::from_utf8;

use super::{read_marker, read_numeric_data};
use super::{
    ReadError,
    MarkerReadError,
};
use super::super::init::Marker;
use super::super::value::ValueRef;

// TODO: Display trait.
#[derive(Debug)]
pub enum Error {
    InvalidBufferFill(ReadError),
    /// Failed to read the marker value.
    InvalidMarkerRead(ReadError),
}

// invalid marker read (IO)
// insuffifient bytes
// invalid string length read (IO)
// length overflow
// invalid utf8

impl From<MarkerReadError> for Error {
    fn from(err: MarkerReadError) -> Error {
        Error::InvalidMarkerRead(From::from(err))
    }
}

// NOTE: Consumes nothing from the given `BufRead` either on success or fail.
pub fn read_value_ref<R>(rd: &mut R) -> Result<ValueRef, Error>
    where R: BufRead
{
    let mut buf = rd.fill_buf().unwrap(); // TODO: May fail.

    // Reading the marker involves either 1 byte read or nothing. On success consumes strictly
    // 1 byte from the `buf`, not from the `rd`.
    let marker = try!(read_marker(&mut buf));

    let val = match marker {
        Marker::Str8 => {
            let len = read_numeric_data::<&[u8], u8>(&mut buf).unwrap(); // TODO: May fail (IO).
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
