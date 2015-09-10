use serde;

use std::fmt;
use std::io::Write;

use rmp::Marker;
use rmp::encode::{
    write_nil,
    write_bool,
    write_uint,
    write_sint,
    write_f32,
    write_f64,
    write_str,
    write_array_len,
    write_map_len,
    write_bin_len,
    WriteError,
    FixedValueWriteError,
    ValueWriteError,
};

#[derive(Debug)]
pub enum Error {
    /// Failed to write MessagePack'ed single-byte value into the write.
    InvalidFixedValueWrite(WriteError),
    InvalidValueWrite(ValueWriteError),

    /// Failed to serialize struct, sequence or map, because its length is unknown.
    UnknownLength,
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidFixedValueWrite(..) => "invalid fixed value write",
            Error::InvalidValueWrite(..) => "invalid value write",
            Error::UnknownLength => "attempt to serialize struct, sequence or map with unknown length",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::InvalidFixedValueWrite(ref err) => Some(err),
            Error::InvalidValueWrite(ref err) => Some(err),
            Error::UnknownLength => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}


impl From<FixedValueWriteError> for Error {
    fn from(err: FixedValueWriteError) -> Error {
        match err {
            FixedValueWriteError(err) => Error::InvalidFixedValueWrite(err)
        }
    }
}

impl From<ValueWriteError> for Error {
    fn from(err: ValueWriteError) -> Error {
        Error::InvalidValueWrite(err)
    }
}

pub trait VariantWriter {
    fn write_struct_len<W>(&self, wr: &mut W, len: u32) -> Result<Marker, ValueWriteError> where W: Write;
    fn write_field_name<W>(&self, wr: &mut W, _key: &str) -> Result<(), ValueWriteError> where W: Write;
}

/// Writes struct as MessagePack array with no field names
pub struct StructArrayWriter;

impl VariantWriter for StructArrayWriter {
    fn write_struct_len<W>(&self, wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
        where W: Write
    {
        write_array_len(wr, len)
    }

    /// This implementation does not write field names
    #[allow(unused_variables)]
    fn write_field_name<W>(&self, wr: &mut W, _key: &str) -> Result<(), ValueWriteError>
        where W: Write
    {
        Ok(())
    }
}

/// Represents MessagePack serialization implementation.
///
/// # Note
///
/// MessagePack has no specification about how to encode variant types. Thus we are free to do
/// whatever we want, so the given chose may be not ideal for you.
///
/// Every Rust variant value can be represented as a tuple of index and a value.
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub struct Serializer<'a, W: VariantWriter> {
    wr: &'a mut Write,
    vw: W,
}

impl<'a> Serializer<'a, StructArrayWriter> {
    /// Creates a new MessagePack encoder whose output will be written to the writer specified.
    pub fn new(wr: &'a mut Write) -> Serializer<'a, StructArrayWriter> {
        Serializer {
            wr: wr,
            vw: StructArrayWriter,
        }
    }
}

impl<'a, W: VariantWriter> Serializer<'a, W> {
    /// Creates a new MessagePack encoder whose output will be written to the writer specified.
    pub fn with(wr: &'a mut Write, vw: W) -> Serializer<'a, W> {
        Serializer {
            wr: wr,
            vw: vw,
        }
    }
}

impl<'a, W: VariantWriter> serde::Serializer for Serializer<'a, W> {
    type Error = Error;

    fn visit_unit(&mut self) -> Result<(), Error> {
        write_nil(&mut self.wr).map_err(From::from)
    }

    fn visit_bool(&mut self, val: bool) -> Result<(), Error> {
        write_bool(&mut self.wr, val).map_err(From::from)
    }

    fn visit_u8(&mut self, val: u8) -> Result<(), Error> {
        self.visit_u64(val as u64)
    }

    fn visit_u16(&mut self, val: u16) -> Result<(), Error> {
        self.visit_u64(val as u64)
    }

    fn visit_u32(&mut self, val: u32) -> Result<(), Error> {
        self.visit_u64(val as u64)
    }

    fn visit_u64(&mut self, val: u64) -> Result<(), Error> {
        try!(write_uint(&mut self.wr, val));

        Ok(())
    }

    fn visit_usize(&mut self, val: usize) -> Result<(), Error> {
        self.visit_u64(val as u64)
    }

    fn visit_i8(&mut self, val: i8) -> Result<(), Error> {
        self.visit_i64(val as i64)
    }

    fn visit_i16(&mut self, val: i16) -> Result<(), Error> {
        self.visit_i64(val as i64)
    }

    fn visit_i32(&mut self, val: i32) -> Result<(), Error> {
        self.visit_i64(val as i64)
    }

    fn visit_i64(&mut self, val: i64) -> Result<(), Error> {
        try!(write_sint(&mut self.wr, val));

        Ok(())
    }

    fn visit_isize(&mut self, val: isize) -> Result<(), Error> {
        self.visit_i64(val as i64)
    }

    fn visit_f32(&mut self, val: f32) -> Result<(), Error> {
        write_f32(&mut self.wr, val).map_err(From::from)
    }

    fn visit_f64(&mut self, val: f64) -> Result<(), Error> {
        write_f64(&mut self.wr, val).map_err(From::from)
    }

    // TODO: The implementation involves heap allocation and is unstable.
    fn visit_char(&mut self, val: char) -> Result<(), Error> {
        let mut buf = String::new();
        buf.push(val);
        self.visit_str(&buf)
    }

    fn visit_str(&mut self, val: &str) -> Result<(), Error> {
        write_str(&mut self.wr, val).map_err(From::from)
    }

    fn visit_unit_variant(&mut self,
                          _name: &str,
                          variant_index: usize,
                          _variant: &str) -> Result<(), Error>
    {
        // Mark that we want to encode a variant type.
        try!(write_array_len(&mut self.wr, 2));

        // Encode a value position...
        try!(self.visit_usize(variant_index));

        // ... and its arguments length.
        try!(write_array_len(&mut self.wr, 0));

        Ok(())
    }

    /// Encodes and attempts to write the enum value into the Write.
    ///
    /// Currently we encode variant types as a tuple of id with array of args, like: [id, [args...]]
    fn visit_tuple_variant<V>(&mut self,
                              _name: &str,
                              variant_index: usize,
                              _variant: &str,
                              mut visitor: V) -> Result<(), Error>
        where V: serde::ser::SeqVisitor,
    {
        // Mark that we want to encode a variant type.
        try!(write_array_len(&mut self.wr, 2));

        // Encode a value position...
        try!(self.visit_usize(variant_index));

        let len = match visitor.len() {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        // ... and its arguments length.
        try!(write_array_len(&mut self.wr, len as u32));

        while let Some(()) = try!(visitor.visit(self)) { }

        Ok(())
    }

    fn visit_struct_variant<V>(&mut self,
                               _name: &str,
                               _variant_index: usize,
                               _variant: &str,
                               _visitor: V) -> Result<(), Error>
        where V: serde::ser::MapVisitor,
    {
        unimplemented!()
    }

    fn visit_none(&mut self) -> Result<(), Error> {
        self.visit_unit()
    }

    fn visit_some<T>(&mut self, v: T) -> Result<(), Error>
        where T: serde::Serialize,
    {
        v.serialize(self)
    }

    // TODO: Check len, overflow is possible.
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: serde::ser::SeqVisitor,
    {
        let len = match visitor.len() {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        try!(write_array_len(&mut self.wr, len as u32));

        while let Some(()) = try!(visitor.visit(self)) { }

        Ok(())
    }

    fn visit_seq_elt<V>(&mut self, value: V) -> Result<(), Error>
        where V: serde::Serialize,
    {
        value.serialize(self)
    }

    fn visit_map<V>(&mut self, mut visitor: V) -> Result<(), Error>
        where V: serde::ser::MapVisitor,
    {
        let len = match visitor.len() {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        try!(write_map_len(&mut self.wr, len as u32));

        while let Some(()) = try!(visitor.visit(self)) { }

        Ok(())
    }

    fn visit_map_elt<K, V>(&mut self, key: K, value: V) -> Result<(), Error>
        where K: serde::Serialize,
              V: serde::Serialize,
    {
        try!(key.serialize(self));
        value.serialize(self)
    }

    fn visit_unit_struct(&mut self, _name: &'static str) -> Result<(), Error> {
        try!(write_array_len(&mut self.wr, 0));

        Ok(())
    }

    fn visit_struct<V>(&mut self, _name: &str, mut visitor: V) -> Result<(), Error>
        where V: serde::ser::MapVisitor,
    {
        let len = match visitor.len() {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        try!(self.vw.write_struct_len(&mut self.wr, len as u32));

        while let Some(()) = try!(visitor.visit(self)) { }

        Ok(())
    }

    fn visit_struct_elt<V>(&mut self, _key: &str, value: V) -> Result<(), Error>
        where V: serde::Serialize,
    {
        try!(self.vw.write_field_name(&mut self.wr, _key));
        value.serialize(self)
    }

    fn visit_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        try!(write_bin_len(&mut self.wr, value.len() as u32));
        self.wr.write_all(value).map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidDataWrite(WriteError(err))))
    }
}
