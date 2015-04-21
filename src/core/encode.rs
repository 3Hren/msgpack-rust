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
    /// Unable to write the value with the given type
    TypeMismatch,
    /// IO error while writing marker.
    InvalidMarkerWrite(WriteError),
    /// IO error while writing single-byte data.
    InvalidFixedValueWrite(WriteError),
    /// IO error while writing data.
    InvalidDataWrite(WriteError),
}

#[unstable(reason = "unwrap")]
fn write_marker<W>(wr: &mut W, marker: Marker) -> Result<(), Error>
    where W: Write
{
    // TODO: Should never panics, but looks creepy. Use own trait instead.
    let byte = marker.to_u8().unwrap();

    match wr.write_u8(byte) {
        Ok(())   => Ok(()),
        Err(err) => Err(Error::InvalidMarkerWrite(From::from(err)))
    }
}

#[unstable(reason = "unwrap")]
fn write_fixval<W>(wr: &mut W, marker: Marker) -> Result<(), Error>
    where W: Write
{
    // TODO: Should never panics, but looks creepy. Use own trait instead.
    let byte = marker.to_u8().unwrap();

    match wr.write_u8(byte) {
        Ok(())   => Ok(()),
        Err(err) => Err(Error::InvalidFixedValueWrite(From::from(err)))
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
        write_fixval(wr, Marker::PositiveFixnum(val))
    } else {
        Err(Error::TypeMismatch)
    }
}

macro_rules! make_write_data_fn {
    (deduce, $writer:ident, $encoder:ident, 0, $val:ident)
        => ($writer.$encoder($val););
    (deduce, $writer:ident, $encoder:ident, 1, $val:ident)
        => ($writer.$encoder::<byteorder::BigEndian>($val););
    (gen, $t:ty, $d:tt, $name:ident, $encoder:ident) => {
        fn $name<W>(wr: &mut W, val: $t) -> Result<(), Error>
            where W: Write
        {
            match make_write_data_fn!(deduce, wr, $encoder, $d, val) {
                Ok(data) => Ok(data),
                Err(err) => Err(Error::InvalidDataWrite(From::from(err))),
            }
        }
    };
    (u8,    $name:ident, $encoder:ident) => (make_write_data_fn!(gen, u8, 0, $name, $encoder););
    (i8,    $name:ident, $encoder:ident) => (make_write_data_fn!(gen, i8, 0, $name, $encoder););
    ($t:ty, $name:ident, $encoder:ident) => (make_write_data_fn!(gen, $t, 1, $name, $encoder););
}

make_write_data_fn!(u8,  write_data_u8,  write_u8);

// With strictly type checking.
pub fn write_u8<W>(wr: &mut W, val: u8) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::U8));

    write_data_u8(wr, val)
}
