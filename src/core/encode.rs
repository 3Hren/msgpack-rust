use std::convert;
use std::io;
use std::io::Write;
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

fn write_marker<W>(wr: &mut W, marker: Marker) -> Result<(), Error>
    where W: Write
{
    let byte = marker.to_u8();

    match wr.write_u8(byte) {
        Ok(())   => Ok(()),
        Err(err) => Err(Error::InvalidMarkerWrite(From::from(err)))
    }
}

fn write_fixval<W>(wr: &mut W, marker: Marker) -> Result<(), Error>
    where W: Write
{
    let byte = marker.to_u8();

    match wr.write_u8(byte) {
        Ok(())   => Ok(()),
        Err(err) => Err(Error::InvalidFixedValueWrite(From::from(err)))
    }
}

/// Unstable: docs; stabilize Result variant
pub fn write_nil<W>(wr: &mut W) -> Result<(), Error>
    where W: Write
{
    write_fixval(wr, Marker::Null)
}

pub fn write_bool<W>(wr: &mut W, val: bool) -> Result<(), Error>
    where W: Write
{
    match val {
        true  => write_fixval(wr, Marker::True),
        false => write_fixval(wr, Marker::False)
    }
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
make_write_data_fn!(f32, write_data_f32, write_f32);
make_write_data_fn!(f64, write_data_f64, write_f64);

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

        write_fixval(wr, marker).and(Ok(marker))
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

pub fn write_f32<W>(wr: &mut W, val: f32) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::F32));
    write_data_f32(wr, val)
}

pub fn write_f64<W>(wr: &mut W, val: f64) -> Result<(), Error>
    where W: Write
{
    try!(write_marker(wr, Marker::F64));
    write_data_f64(wr, val)
}

/// Writes the most efficient string length implementation to the given buffer.
///
/// This function is useful when you want to get full control for writing the data itself, for
/// example, when using non-blocking socket.
pub fn write_str_len<W>(wr: &mut W, len: u32) -> Result<Marker, Error>
    where W: Write
{
    if len < 32 {
        let marker = Marker::FixedString(len as u8);
        write_fixval(wr, marker).map(|_| marker)
    } else if len < 256 {
        try!(write_marker(wr, Marker::Str8));
        write_data_u8(wr, len as u8).map(|_| Marker::Str8)
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Str16));
        write_data_u16(wr, len as u16).map(|_| Marker::Str16)
    } else {
        try!(write_marker(wr, Marker::Str32));
        write_data_u32(wr, len).map(|_| Marker::Str32)
    }
}

pub fn write_bin_len<W>(wr: &mut W, len: u32) -> Result<Marker, Error>
    where W: Write
{
    if len < 256 {
        try!(write_marker(wr, Marker::Bin8));
        write_data_u8(wr, len as u8).map(|_| Marker::Bin8)
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Bin16));
        write_data_u16(wr, len as u16).map(|_| Marker::Bin16)
    } else {
        try!(write_marker(wr, Marker::Bin32));
        write_data_u32(wr, len).map(|_| Marker::Bin32)
    }
}

pub fn write_array_len<W>(wr: &mut W, len: u32) -> Result<Marker, Error>
    where W: Write
{
    if len < 16 {
        let marker = Marker::FixedArray(len as u8);
        write_fixval(wr, marker).map(|_| marker)
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Array16));
        write_data_u16(wr, len as u16).map(|_| Marker::Array16)
    } else {
        try!(write_marker(wr, Marker::Array32));
        write_data_u32(wr, len).map(|_| Marker::Array32)
    }
}

pub fn write_map_len<W>(wr: &mut W, len: u32) -> Result<Marker, Error>
    where W: Write
{
    if len < 16 {
        let marker = Marker::FixedMap(len as u8);
        write_fixval(wr, marker).map(|_| marker)
    } else if len < 65536 {
        try!(write_marker(wr, Marker::Map16));
        write_data_u16(wr, len as u16).map(|_| Marker::Map16)
    } else {
        try!(write_marker(wr, Marker::Map32));
        write_data_u32(wr, len).map(|_| Marker::Map32)
    }
}

/// typeid < 0 is reserved for future extension including 2-byte type information.
pub fn write_ext_meta<W>(wr: &mut W, len: u32, typeid: i8) -> Result<Marker, Error>
    where W: Write
{
    match len {
        1 => {
            try!(write_marker(wr, Marker::FixExt1));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::FixExt1)
        }
        2 => {
            try!(write_marker(wr, Marker::FixExt2));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::FixExt2)
        }
        4 => {
            try!(write_marker(wr, Marker::FixExt4));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::FixExt4)
        }
        8 => {
            try!(write_marker(wr, Marker::FixExt8));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::FixExt8)
        }
        16 => {
            try!(write_marker(wr, Marker::FixExt16));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::FixExt16)
        }
        len if len < 256 => {
            try!(write_marker(wr, Marker::Ext8));
            try!(write_data_u8(wr, len as u8));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::Ext8)
        }
        len if len < 65536 => {
            try!(write_marker(wr, Marker::Ext16));
            try!(write_data_u16(wr, len as u16));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::Ext16)
        }
        len => {
            try!(write_marker(wr, Marker::Ext32));
            try!(write_data_u32(wr, len));
            try!(write_data_i8(wr, typeid));
            Ok(Marker::Ext32)
        }
    }
}

pub mod serialize {

use serialize;

use std::io;
use std::io::Write;

use super::{
    write_nil,
    write_bool,
    write_uint,
    write_sint,
    write_f32,
    write_f64,
    write_str_len,
    write_array_len,
    write_map_len,
};

pub enum Error {
    Unimplemented,
}

impl From<super::Error> for Error {
    fn from(err: super::Error) -> Error {
        match err {
            _ => Error::Unimplemented,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        match err {
            _ => Error::Unimplemented,
        }
    }
}

pub struct Encoder<'a> {
    wr: &'a mut Write,
}

impl<'a> Encoder<'a> {
    pub fn new(wr: &'a mut Write) -> Encoder<'a> {
        Encoder {
            wr: wr,
        }
    }
}

#[allow(unused)]
impl<'a> serialize::Encoder for Encoder<'a> {
    type Error = Error;

    fn emit_nil(&mut self) -> Result<(), Error> {
        write_nil(&mut self.wr).map_err(|err| From::from(err))
    }

    fn emit_bool(&mut self, val: bool) -> Result<(), Error> {
        write_bool(&mut self.wr, val).map_err(|err| From::from(err))
    }

    fn emit_u8(&mut self, val: u8) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u16(&mut self, val: u16) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u32(&mut self, val: u32) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u64(&mut self, val: u64) -> Result<(), Error> {
        try!(write_uint(&mut self.wr, val));

        Ok(())
    }

    fn emit_usize(&mut self, val: usize) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_i8(&mut self, val: i8) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i16(&mut self, val: i16) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i32(&mut self, val: i32) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i64(&mut self, val: i64) -> Result<(), Error> {
        try!(write_sint(&mut self.wr, val));

        Ok(())
    }

    fn emit_isize(&mut self, val: isize) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_f32(&mut self, val: f32) -> Result<(), Error> {
        write_f32(&mut self.wr, val).map_err(|err| From::from(err))
    }

    fn emit_f64(&mut self, val: f64) -> Result<(), Error> {
        write_f64(&mut self.wr, val).map_err(|err| From::from(err))
    }

    // TODO: The implementation involves heap allocation and is unstable.
    fn emit_char(&mut self, val: char) -> Result<(), Error> {
        let mut buf = String::new();
        buf.push(val);
        self.emit_str(&buf)
    }

    fn emit_str(&mut self, val: &str) -> Result<(), Error> {
        try!(write_str_len(&mut self.wr, val.len() as u32));
        // TODO: Implement this functionality in the low-level module.
        try!(self.wr.write_all(val.as_bytes()));

        Ok(())
    }

    fn emit_enum<F>(&mut self, name: &str, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_enum_variant<F>(&mut self, v_name: &str, v_id: usize, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_enum_variant_arg<F>(&mut self, a_idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_enum_struct_variant<F>(&mut self, v_name: &str, v_id: usize, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_enum_struct_variant_field<F>(&mut self, f_name: &str, f_idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_struct<F>(&mut self, _name: &str, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        self.emit_tuple(len, f)
    }

    fn emit_struct_field<F>(&mut self, _name: &str, idx_: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_array_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_tuple_arg<F>(&mut self, idx_: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_tuple_struct<F>(&mut self, name_: &str, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        self.emit_tuple(len, f)
    }

    fn emit_tuple_struct_arg<F>(&mut self, idx_: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_option<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_option_none(&mut self) -> Result<(), Error> {
        self.emit_nil()
    }

    fn emit_option_some<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    // TODO: Check len, overflow is possible.
    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_array_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_seq_elt<F>(&mut self, idx_: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_map_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_map_elt_key<F>(&mut self, idx_: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_map_elt_val<F>(&mut self, idx_: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }
}

}
