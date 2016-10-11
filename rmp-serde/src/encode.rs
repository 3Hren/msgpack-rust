use serde;

use std::fmt;
use std::io::Write;

use rmp::Marker;
use rmp::encode::{
    write_nil,
    write_bool,
    write_uint,
    write_sint_eff,
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

    /// Depth limit exceeded
    DepthLimitExceeded,
    Custom(String)
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidFixedValueWrite(..) => "invalid fixed value write",
            Error::InvalidValueWrite(..) => "invalid value write",
            Error::UnknownLength => "attempt to serialize struct, sequence or map with unknown length",
            Error::DepthLimitExceeded => "depth limit exceeded",
            Error::Custom(..) => "custom message",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::InvalidFixedValueWrite(ref err) => Some(err),
            Error::InvalidValueWrite(ref err) => Some(err),
            Error::UnknownLength => None,
            Error::DepthLimitExceeded => None,
            Error::Custom(_) => None,
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

impl serde::ser::Error for Error {
    /// Raised when there is general error when deserializing a type.
    fn custom<T: Into<String>>(msg: T) -> Error {
        Error::Custom(msg.into())
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
            depth: 1000,
        }
    }
}

impl<'a, W: VariantWriter> Serializer<'a, W> {
    /// Creates a new MessagePack encoder whose output will be written to the writer specified.
    pub fn with(wr: &'a mut Write, vw: W) -> Serializer<'a, W> {
        Serializer {
            wr: wr,
            vw: vw,
            depth: 1000,
        }
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

    #[inline]
    fn serialize_unit(&mut self) -> Result<(), Error> {
        write_nil(&mut self.wr).map_err(From::from)
    }

    #[inline]
    fn serialize_bool(&mut self, val: bool) -> Result<(), Error> {
        write_bool(&mut self.wr, val).map_err(From::from)
    }

    #[inline]
    fn serialize_u8(&mut self, val: u8) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    #[inline]
    fn serialize_u16(&mut self, val: u16) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    #[inline]
    fn serialize_u32(&mut self, val: u32) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    #[inline]
    fn serialize_u64(&mut self, val: u64) -> Result<(), Error> {
        try!(write_uint(&mut self.wr, val));

        Ok(())
    }

    #[inline]
    fn serialize_usize(&mut self, val: usize) -> Result<(), Error> {
        self.serialize_u64(val as u64)
    }

    #[inline]
    fn serialize_i8(&mut self, val: i8) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    #[inline]
    fn serialize_i16(&mut self, val: i16) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    #[inline]
    fn serialize_i32(&mut self, val: i32) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    #[inline]
    fn serialize_i64(&mut self, val: i64) -> Result<(), Error> {
        try!(write_sint_eff(&mut self.wr, val));

        Ok(())
    }

    #[inline]
    fn serialize_isize(&mut self, val: isize) -> Result<(), Error> {
        self.serialize_i64(val as i64)
    }

    #[inline]
    fn serialize_f32(&mut self, val: f32) -> Result<(), Error> {
        write_f32(&mut self.wr, val).map_err(From::from)
    }

    #[inline]
    fn serialize_f64(&mut self, val: f64) -> Result<(), Error> {
        write_f64(&mut self.wr, val).map_err(From::from)
    }

    // TODO: The implementation involves heap allocation and is unstable.
    #[inline]
    fn serialize_char(&mut self, val: char) -> Result<(), Error> {
        let mut buf = String::new();
        buf.push(val);
        self.serialize_str(&buf)
    }

    #[inline]
    fn serialize_str(&mut self, val: &str) -> Result<(), Error> {
        write_str(&mut self.wr, val).map_err(From::from)
    }

    #[inline]
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

        Ok(())
    }

    /// Encodes and attempts to write the enum value into the Write.
    ///
    /// Currently we encode variant types as a tuple of id with array of args, like: [id, [args...]]
    #[inline]
    fn serialize_tuple_variant(&mut self,
                               _name: &str,
                               variant_index: usize,
                               _variant: &str,
                               len: usize)
                               -> Result<(), Error> {
        let mut state = try!(self.serialize_seq(Some(2)));

        try!(self.serialize_seq_elt(&mut state, variant_index));
        try!(self.serialize_seq(Some(len)));

        self.serialize_seq_end(state)
    }

    #[inline]
    fn serialize_tuple_variant_elt<T>(&mut self,
                                      state: &mut Self::TupleVariantState,
                                      value: T)
                                      -> Result<Self::TupleVariantState, Self::Error>
        where T: serde::Serialize
    {
        try!(value.serialize(self));
        Ok(*state)
    }

    #[inline]
    fn serialize_tuple_variant_end(&mut self,
                                   _state: Self::TupleVariantState)
                                   -> Result<(), Self::Error> {
        Ok(())
    }

    fn serialize_struct_variant(&mut self,
                                _name: &str,
                                variant_index: usize,
                                _variant: &str,
                                len: usize)
                                -> Result<Self::StructState, Error> {
        let mut state = try!(self.serialize_seq(Some(2)));

        try!(self.serialize_seq_elt(&mut state, variant_index));

        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant_elt<V>(&mut self,
                                       state: &mut Self::StructVariantState,
                                       key: &'static str,
                                       value: V)
                                       -> Result<Self::StructVariantState, Self::Error>
        where V: serde::Serialize
    {
        self.serialize_struct_elt(state, key, value)
    }

    #[inline]
    fn serialize_struct_variant_end(&mut self,
                                    _state: Self::StructVariantState)
                                    -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_none(&mut self) -> Result<(), Error> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(&mut self, v: T) -> Result<(), Error>
        where T: serde::Serialize,
    {
        depth_count!(self.depth, v.serialize(self))
    }

    // TODO: Check len, overflow is possible.
    #[inline]
    fn serialize_seq(&mut self, len: Option<usize>) -> Result<Self::StructState, Error> {
        let len = match len {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        try!(write_array_len(&mut self.wr, len as u32));

        Ok(())
    }

    #[inline]
    fn serialize_seq_fixed_size(&mut self, size: usize) -> Result<Self::SeqState, Self::Error> {
        self.serialize_seq(Some(size))
    }

    #[inline]
    fn serialize_seq_elt<T>(&mut self,
                            _state: &mut Self::SeqState,
                            value: T)
                            -> Result<Self::SeqState, Self::Error>
        where T: serde::Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_seq_end(&mut self, _state: Self::SeqState) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_tuple(&mut self, len: usize) -> Result<Self::TupleState, Self::Error> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_elt<T>(&mut self,
                              state: &mut Self::TupleState,
                              value: T)
                              -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_seq_elt(state, value)
    }

    #[inline]
    fn serialize_tuple_end(&mut self, state: Self::TupleState) -> Result<(), Self::Error> {
        self.serialize_seq_end(state)
    }

    #[inline]
    fn serialize_tuple_struct(&mut self,
                              _name: &'static str,
                              len: usize)
                              -> Result<(), Self::Error> {
        self.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_struct_elt<T>(&mut self,
                                     state: &mut Self::TupleStructState,
                                     value: T)
                                     -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_tuple_elt(state, value)
    }

    #[inline]
    fn serialize_tuple_struct_end(&mut self,
                                  state: Self::TupleStructState)
                                  -> Result<(), Self::Error> {
        self.serialize_tuple_end(state)
    }

    #[inline]
    fn serialize_map(&mut self, len: Option<usize>) -> Result<Self::MapState, Self::Error> {
        let len = match len {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        try!(write_map_len(&mut self.wr, len as u32));

        Ok(())
    }

    #[inline]
    fn serialize_map_key<T>(&mut self,
                            _state: &mut Self::MapState,
                            key: T)
                            -> Result<Self::MapState, Self::Error>
        where T: serde::Serialize
    {
        key.serialize(self)
    }

    #[inline]
    fn serialize_map_value<T>(&mut self,
                              _state: &mut Self::MapState,
                              value: T)
                              -> Result<Self::MapState, Self::Error>
        where T: serde::Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_map_end(&mut self, _state: Self::MapState) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(&mut self, _name: &'static str) -> Result<(), Error> {
        try!(self.vw.write_struct_len(&mut self.wr, 0));

        Ok(())
    }

    #[inline]
    fn serialize_newtype_struct<T>(&mut self,
                                   _name: &'static str,
                                   value: T)
                                   -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(&mut self,
                                    name: &'static str,
                                    variant_index: usize,
                                    variant: &'static str,
                                    _value: T)
                                    -> Result<(), Self::Error>
        where T: serde::Serialize
    {
        self.serialize_tuple_variant(name, variant_index, variant, 1)
    }

    #[inline]
    fn serialize_struct(&mut self,
                        _name: &'static str,
                        len: usize)
                        -> Result<Self::StructState, Self::Error> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_elt<V>(&mut self,
                               state: &mut Self::StructState,
                               key: &'static str,
                               value: V)
                               -> Result<Self::StructState, Self::Error>
        where V: serde::Serialize
    {
        try!(self.serialize_map_key(state, key));
        self.serialize_map_value(state, value)
    }

    #[inline]
    fn serialize_struct_end(&mut self,
                            state: Self::StructState)
                            -> Result<Self::StructState, Error> {
        self.serialize_map_end(state)
    }

    #[inline]
    fn serialize_bytes(&mut self, value: &[u8]) -> Result<(), Error> {
        try!(write_bin_len(&mut self.wr, value.len() as u32));
        self.wr.write_all(value).map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidDataWrite(WriteError(err))))
    }
}
