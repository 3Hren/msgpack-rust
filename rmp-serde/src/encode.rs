use std::error;
use std::fmt::{self, Display};
use std::io::Write;

use serde;

use rmp::Marker;
use rmp::encode::{write_nil, write_bool, write_uint, write_sint, write_f32, write_f64, write_str,
                  write_array_len, write_map_len, write_bin_len, ValueWriteError};

#[derive(Debug)]
pub enum Error {
    InvalidValueWrite(ValueWriteError),

    /// Failed to serialize struct, sequence or map, because its length is unknown.
    UnknownLength,

    /// Depth limit exceeded
    DepthLimitExceeded,
    Custom(String)
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidValueWrite(..) => "invalid value write",
            Error::UnknownLength => "attempt to serialize struct, sequence or map with unknown length",
            Error::DepthLimitExceeded => "depth limit exceeded",
            Error::Custom(..) => "custom message",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidValueWrite(ref err) => Some(err),
            Error::UnknownLength => None,
            Error::DepthLimitExceeded => None,
            Error::Custom(..) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(f)
    }
}

impl From<ValueWriteError> for Error {
    fn from(err: ValueWriteError) -> Error {
        Error::InvalidValueWrite(err)
    }
}

impl serde::ser::Error for Error {
    /// Raised when there is general error when deserializing a type.
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Custom(msg.into())
    }
}

pub trait VariantWriter {
    fn write_struct_len<W>(&self, wr: &mut W, len: u32) -> Result<Marker, ValueWriteError>
        where W: Write;
    fn write_field_name<W>(&self, wr: &mut W, _key: &str) -> Result<(), ValueWriteError>
        where W: Write;
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
    depth: usize,
}

impl<'a, W: VariantWriter> Serializer<'a, W> {
    /// Changes the maximum nesting depth that is allowed
    pub fn set_max_depth(&mut self, depth: usize) {
        self.depth = depth;
    }
}

macro_rules! depth_count(
    ( $counter:expr, $expr:expr ) => {
        {
            $counter -= 1;
            if $counter == 0 {
                return Err(Error::DepthLimitExceeded)
            }
            let res = $expr;
            $counter += 1;
            res
        }
    }
);

impl<'a> Serializer<'a, StructArrayWriter> {
    /// Creates a new MessagePack encoder whose output will be written to the writer specified.
    pub fn new(wr: &'a mut Write) -> Serializer<'a, StructArrayWriter> {
        Serializer {
            wr: wr,
            vw: StructArrayWriter,
            depth: 1024,
        }
    }
}

impl<'a, W: VariantWriter> Serializer<'a, W> {
    /// Creates a new MessagePack encoder whose output will be written to the writer specified.
    pub fn with(wr: &'a mut Write, vw: W) -> Serializer<'a, W> {
        Serializer {
            wr: wr,
            vw: vw,
            depth: 1024,
        }
    }

    ///Follow with standard seq style for variables
    #[inline]
    fn serialize_variant(&mut self, variant_index: usize, maybe_len: Option<usize>) -> Result<(), Error>
    {
        try!(write_array_len(&mut self.wr, 2));

        // Encode a value position...
        try!(serde::Serializer::serialize_usize(self, variant_index));

        let len = match maybe_len {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        // ... and its arguments length.
        try!(write_array_len(&mut self.wr, len as u32));

        Ok(())
    }
}

impl<'a, W: VariantWriter> serde::Serializer for Serializer<'a, W> {
    type Error = Error;
    type SeqState = ();
    type TupleState = ();
    type TupleStructState = ();
    type TupleVariantState = ();
    type MapState = ();
    type StructState = ();
    type StructVariantState = ();

    fn serialize_unit(&mut self) -> Result<(), Error> {
        write_nil(&mut self.wr)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(err)))
    }

    fn serialize_bool(&mut self, val: bool) -> Result<(), Error> {
        write_bool(&mut self.wr, val)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(err)))
    }

    fn serialize_u8(&mut self, val: u8) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    fn serialize_u16(&mut self, val: u16) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    fn serialize_u32(&mut self, val: u32) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    fn serialize_u64(&mut self, val: u64) -> Result<(), Error> {
        try!(write_uint(&mut self.wr, val));
        Ok(())
    }

    fn serialize_usize(&mut self, val: usize) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    fn serialize_i8(&mut self, val: i8) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_i16(&mut self, val: i16) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_i32(&mut self, val: i32) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_i64(&mut self, val: i64) -> Result<(), Error> {
        try!(write_sint(&mut self.wr, val));
        Ok(())
    }

    fn serialize_isize(&mut self, val: isize) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    fn serialize_f32(&mut self, val: f32) -> Result<(), Error> {
        write_f32(&mut self.wr, val).map_err(From::from)
    }

    fn serialize_f64(&mut self, val: f64) -> Result<(), Error> {
        write_f64(&mut self.wr, val).map_err(From::from)
    }

    // TODO: The implementation involves heap allocation and is unstable.
    fn serialize_char(&mut self, val: char) -> Result<(), Error> {
        let mut buf = String::new();
        buf.push(val);
        self.serialize_str(&buf)
    }

    fn serialize_str(&mut self, val: &str) -> Result<(), Error> {
        write_str(&mut self.wr, val).map_err(From::from)
    }

    fn serialize_unit_variant(&mut self,
                          _name: &str,
                          variant_index: usize,
                          _variant: &str) -> Result<(), Error>
    {
        // Mark that we want to encode a variant type.
        try!(write_array_len(&mut self.wr, 2));

        // Encode a value position...
        try!(self.serialize_usize(variant_index));

        // ... and its arguments length.
        try!(write_array_len(&mut self.wr, 0));

        try!(write_nil(&mut self.wr)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(err))));

        Ok(())
    }

    /// Encodes and attempts to write the enum value into the Write.
    ///
    /// Currently we encode variant types as a tuple of id with array of args, like: [id, [args...]]
    fn serialize_tuple_variant(&mut self, name: &'static str, variant_index: usize, _variant : &'static str, len: usize) ->
        Result<Self::TupleVariantState, Self::Error>
    {
        self.serialize_variant(variant_index, Some(len))
        .and_then(|_| self.serialize_tuple_struct(name, len))
    }

    fn serialize_tuple_variant_elt<T>(&mut self, state: &mut Self::TupleVariantState, value: T) -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_tuple_struct_elt(state, value)
    }

    fn serialize_tuple_variant_end(&mut self, state: Self::TupleVariantState) -> Result<(), Self::Error> {
        self.serialize_tuple_struct_end(state)
    }

    fn serialize_none(&mut self) -> Result<(), Error> {
        self.serialize_unit()
    }

    fn serialize_some<V>(&mut self, v: V) -> Result<(), Error>
        where V: serde::Serialize,
    {
        v.serialize(self)
    }

    // TODO: Check len, overflow is possible.
    fn serialize_seq(&mut self, length: Option<usize>) -> Result<Self::SeqState, Error>
    {
        let len = match length {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        try!(write_array_len(&mut self.wr, len as u32));

        Ok(())
    }

    fn serialize_seq_elt<V>(&mut self, _state: &mut Self::SeqState, value: V) -> Result<Self::SeqState, Error>
        where V: serde::Serialize
    {
        let _ = value.serialize(self);
        Ok(())
    }

    fn serialize_seq_end(&mut self, _state: Self::SeqState) -> Result<(), Self::Error>
    {
        Ok(())
    }

    fn serialize_seq_fixed_size(&mut self, size: usize) -> Result<Self::SeqState, Self::Error>
    {
        self.serialize_seq(Some(size))
    }

    fn serialize_map(&mut self, length: Option<usize>) -> Result<Self::MapState, Error>
    {
        let len = match length {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        try!(write_map_len(&mut self.wr, len as u32));

        Ok(())
    }

    fn serialize_map_key<T>(&mut self, _state: &mut Self::MapState, key: T) -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        let _ = key.serialize(self);

        Ok(())
    }

    fn serialize_map_value<T>(&mut self, _state: &mut Self::MapState, value: T) -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        let _ = value.serialize(self);

        Ok(())
    }

    fn serialize_map_end(&mut self, _state: Self::MapState) -> Result<(), Self::Error>
    {
        Ok(())
    }

    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<(), Error>
    {
        try!(self.vw.write_struct_len(&mut self.wr, 0));

        Ok(())
    }

    fn serialize_tuple(&mut self, len: usize) -> Result<Self::TupleState, Self::Error>
    {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_elt<T>(&mut self, state: &mut Self::TupleState, value: T) -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_seq_elt(state, value)
    }

    fn serialize_tuple_end(&mut self, state: Self::TupleState) -> Result<(), Self::Error>
    {
        self.serialize_seq_end(state)
    }

    fn serialize_tuple_struct(&mut self, _name: &'static str, len: usize) -> Result<Self::TupleStructState, Self::Error>
    {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_struct_elt<T>(&mut self, state: &mut Self::TupleStructState, value: T) -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_tuple_elt(state, value)
    }

    fn serialize_tuple_struct_end(&mut self, state: Self::TupleStructState) -> Result<(), Self::Error>
    {
        self.serialize_tuple_end(state)
    }

    fn serialize_newtype_struct<T>(&mut self, name: &'static str, value: T) -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_tuple_struct(name, 1)
        .and_then(|mut state: Self::TupleState| self.serialize_tuple_struct_elt(&mut state, value))
        .and_then(|state: Self::TupleState| self.serialize_tuple_struct_end(state))
    }

    fn serialize_newtype_variant<T>(&mut self, name: &'static str, variant_index: usize, variant: &'static str, value: T) -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_tuple_variant(name, variant_index, variant, 1)
        .and_then(|mut state: Self::TupleState| self.serialize_tuple_variant_elt(&mut state, value))
        .and_then(|state: Self::TupleState| self.serialize_tuple_variant_end(state))
    }

    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        try!(write_bin_len(&mut self.wr, value.len() as u32));
        self.wr.write_all(value)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidDataWrite(err)))
    }

    /// Begins to serialize a struct. This call must be followed by zero or more
    /// calls to `serialize_struct_elt`, then a call to `serialize_struct_end`.
    fn serialize_struct(&mut self, _name: &'static str, len: usize) -> Result<Self::StructState, Self::Error>
    {
        try!(self.vw.write_struct_len(&mut self.wr, len as u32));
        Ok(())
    }

    /// Serializes a struct field. Must have previously called
    /// `serialize_struct`.
    fn serialize_struct_elt<V>(&mut self, _state: &mut Self::StructState, key: &'static str, value: V) ->  Result<(), Self::Error>
        where V: serde::Serialize
    {
        self.vw.write_field_name(&mut self.wr, key)
            .map_err(|e| e.into())
            .and_then(|_| value.serialize(self))
            .map(|_| ())
    }

    /// Finishes serializing a struct.
    fn serialize_struct_end(&mut self, _state: Self::StructState) -> Result<(), Self::Error>
    {
        Ok(())
    }

    /// Begins to serialize a struct variant. This call must be followed by zero
    /// or more calls to `serialize_struct_variant_elt`, then a call to
    /// `serialize_struct_variant_end`.
    fn serialize_struct_variant(&mut self, name: &'static str, variant_index: usize, _variant: &'static str, len: usize) ->
        Result<Self::StructVariantState, Self::Error>
    {
        let _ = self.serialize_variant(variant_index, Some(len));
        self.serialize_struct(name, len)
    }

    /// Serialize a struct variant element. Must have previously called
    /// `serialize_struct_variant`.
    fn serialize_struct_variant_elt<V>(&mut self, state: &mut Self::StructVariantState, key: &'static str, value: V) -> Result<(), Self::Error>
        where V: serde::Serialize
    {
        self.serialize_struct_elt(state, key, value)
    }

    /// Finishes serializing a struct variant.
    fn serialize_struct_variant_end(&mut self,state: Self::StructVariantState) -> Result<(), Self::Error>
    {
        self.serialize_struct_end(state)
    }
}
