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

// TODO: Split Error for each function, permitting each function to return variant with impossible values.
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

pub fn write_nfix<W>(wr: &mut W, val: i8) -> Result<(), Error>
    where W: Write
{
    if -32 <= val && val < 0 {
        write_fixval(wr, Marker::NegativeFixnum(val))
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
make_write_data_fn!(u16, write_data_u16, write_u16);
make_write_data_fn!(u32, write_data_u32, write_u32);
make_write_data_fn!(u64, write_data_u64, write_u64);
make_write_data_fn!(i8,  write_data_i8,  write_i8);
make_write_data_fn!(i16, write_data_i16, write_i16);
make_write_data_fn!(i32, write_data_i32, write_i32);
make_write_data_fn!(i64, write_data_i64, write_i64);

// With strictly type checking.
pub fn write_u8<W>(wr: &mut W, val: u8) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::U8));
    write_data_u8(wr, val)
}

pub fn write_u16<W>(wr: &mut W, val: u16) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::U16));
    write_data_u16(wr, val)
}

pub fn write_u32<W>(wr: &mut W, val: u32) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::U32));
    write_data_u32(wr, val)
}

pub fn write_u64<W>(wr: &mut W, val: u64) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::U64));
    write_data_u64(wr, val)
}

pub fn write_i8<W>(wr: &mut W, val: i8) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::I8));
    write_data_i8(wr, val)
}

pub fn write_i16<W>(wr: &mut W, val: i16) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::I16));
    write_data_i16(wr, val)
}

pub fn write_i32<W>(wr: &mut W, val: i32) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::I32));
    write_data_i32(wr, val)
}

pub fn write_i64<W>(wr: &mut W, val: i64) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::I64));
    write_data_i64(wr, val)
}

/// [Write Me].
///
/// According to the MessagePack specification, the serializer SHOULD use the format which
/// represents the data in the smallest number of bytes.
pub fn write_uint<W>(wr: &mut W, val: u64) -> Result<Marker, Error>
    where W: Write
{
    if val < 128 {
        let marker = Marker::PositiveFixnum(val as u8);

        write_fixval(wr, marker).map(|_| marker)
    } else if val < 256 {
        write_u8(wr, val as u8).map(|_| Marker::U8)
    } else if val < 65536 {
        write_u16(wr, val as u16).map(|_| Marker::U16)
    } else if val < 4294967296 {
        write_u32(wr, val as u32).map(|_| Marker::U32)
    } else {
        write_u64(wr, val).map(|_| Marker::U64)
    }
}

/// [Write Me].
///
/// According to the MessagePack specification, the serializer SHOULD use the format which
/// represents the data in the smallest number of bytes.
pub fn write_sint<W>(wr: &mut W, val: i64) -> Result<Marker, Error>
    where W: Write
{
    if -32 <= val && val <= 0 {
        let marker = Marker::NegativeFixnum(val as i8);

        write_fixval(wr, marker).map(|_| marker)
    } else if -128 <= val && val < 128 {
        write_i8(wr, val as i8).map(|_| Marker::I8)
    } else if -32768 <= val && val < 32768 {
        write_i16(wr, val as i16).map(|_| Marker::I16)
    } else if -2147483648 <= val && val <= 2147483647 {
        write_i32(wr, val as i32).map(|_| Marker::I32)
    } else {
        write_i64(wr, val).map(|_| Marker::I64)
    }
}
