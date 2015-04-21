use std::convert;
use std::io;
use std::io::Write;
use std::num::ToPrimitive;
use std::result::Result;

use byteorder;
use byteorder::WriteBytesExt;

use super::{
    Marker,
};

#[derive(Debug)]
pub enum WriteError {
    Io(io::Error),
}

impl convert::From<byteorder::Error> for WriteError {
    fn from(err: byteorder::Error) -> WriteError {
        match err {
            byteorder::Error::UnexpectedEOF => unimplemented!(),
            byteorder::Error::Io(err) => WriteError::Io(err),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    /// IO error while writing marker.
    InvalidMarkerWrite(WriteError),
//    InvalidFixedValueWrite(io::Error),
//    InvalidDataWrite(io::Error),
//    IntegerTooLarge,
}

fn write_marker<W>(wr: &mut W, marker: Marker) -> Result<(), Error>
    where W: Write
{
    let byte = marker.to_u8().unwrap();

    match wr.write_u8(byte) {
        Ok(())   => Ok(()),
        Err(err) => Err(Error::InvalidMarkerWrite(From::from(err)))
    }
}

#[unstable(reason = "docs; stabilize Result variant")]
pub fn write_nil<W>(wr: &mut W) -> Result<(), Error>
    where W: Write
{
    write_marker(wr, Marker::Null)
}

// With strictly type checking.
pub fn write_pfix<W>(wr: &mut W, val: u8) -> Result<(), Error>
    where W: Write
{
    if val < 128 {
        write_marker(wr, Marker::PositiveFixnum(val))
    } else {
        unimplemented!()
    }
}
