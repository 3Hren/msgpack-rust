//! Serialize a Rust data structure into MessagePack data.

use std::error;
use std::fmt::{self, Display};
use std::io::Write;

use serde;
use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
                 SerializeTuple, SerializeTupleStruct, SerializeTupleVariant};

use rmp::encode;
use rmp::encode::ValueWriteError;

use ext::{StructMapSerializer, StructTupleSerializer};

/// This type represents all possible errors that can occur when serializing or
/// deserializing MessagePack data.
#[derive(Debug)]
pub enum Error {
    /// Failed to write a MessagePack value.
    InvalidValueWrite(ValueWriteError),
    /// Failed to serialize struct, sequence or map, because its length is unknown.
    UnknownLength,
    /// Depth limit exceeded
    DepthLimitExceeded,
    /// Catchall for syntax error messages.
    Syntax(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidValueWrite(..) => "invalid value write",
            Error::UnknownLength => {
                "attempt to serialize struct, sequence or map with unknown length"
            }
            Error::DepthLimitExceeded => "depth limit exceeded",
            Error::Syntax(..) => "syntax error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::InvalidValueWrite(ref err) => Some(err),
            Error::UnknownLength => None,
            Error::DepthLimitExceeded => None,
            Error::Syntax(..) => None,
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
    fn custom<T: Display>(msg: T) -> Error {
        Error::Syntax(msg.to_string())
    }
}

/// Obtain the underlying writer.
pub trait UnderlyingWrite {
    /// Underlying writer type.
    type Write: Write;

    /// Gets a reference to the underlying writer.
    fn get_ref(&self) -> &Self::Write;

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    fn get_mut(&mut self) -> &mut Self::Write;

    /// Unwraps this `Serializer`, returning the underlying writer.
    fn into_inner(self) -> Self::Write;
}

/// Represents MessagePack serialization implementation.
///
/// # Note
///
/// MessagePack has no specification about how to encode enum types. Thus we are free to do
/// whatever we want, so the given chose may be not ideal for you.
///
/// An enum value is represented as a single-entry map whose key is the variant
/// id and whose value is a sequence containing all associated data. If the enum
/// does not have associated data, the sequence is empty.
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
#[derive(Debug)]
pub struct Serializer<W> {
    wr: W,
    depth: usize,
}

impl<W: Write> Serializer<W> {
    /// Gets a reference to the underlying writer.
    pub fn get_ref(&self) -> &W {
        &self.wr
    }

    /// Gets a mutable reference to the underlying writer.
    ///
    /// It is inadvisable to directly write to the underlying writer.
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.wr
    }

    /// Unwraps this `Serializer`, returning the underlying writer.
    pub fn into_inner(self) -> W {
        self.wr
    }

    /// Changes the maximum nesting depth that is allowed.
    ///
    /// Currently unused.
    #[doc(hidden)]
    pub fn set_max_depth(&mut self, depth: usize) {
        self.depth = depth;
    }

    /// Constructs a new `MessagePack` serializer whose output will be written to the writer
    /// specified.
    ///
    /// # Note
    ///
    /// This is the default constructor, which returns a serializer that will serialize structs
    /// and enums using the most compact representation.
    pub fn new(wr: W) -> Self {
        Serializer {
            wr: wr,
            depth: 1024,
        }
    }

    #[deprecated(note = "use `Serializer::new` instead")]
    #[doc(hidden)]
    pub fn compact(wr: W) -> Self {
        Serializer {
            wr: wr,
            depth: 1024,
        }
    }

    #[deprecated(note = "use `Serializer::with_struct_map()` instead")]
    #[doc(hidden)]
    pub fn new_named(wr: W) -> Self {
        Serializer {
            wr: wr,
            depth: 1024,
        }
    }
}

impl<'a, W: Write + 'a> Serializer<W> {
    #[inline]
    fn compound(&'a mut self) -> Result<Compound<'a, W>, Error> {
        let c = Compound { se: self };
        Ok(c)
    }
}

/// Extends the serializer by allowing to generate extension wrappers.
pub trait Ext: UnderlyingWrite + Sized {
    /// Consumes this serializer returning the new one, which will serialize structs as a map.
    ///
    /// This is used, when you the default struct serialization as a tuple does not fit your
    /// requirements.
    fn with_struct_map(self) -> StructMapSerializer<Self> {
        StructMapSerializer::new(self)
    }

    /// Consumes this serializer returning the new one, which will serialize structs as a tuple
    /// without field names.
    ///
    /// This is the default MessagePack serialization mechanism, emitting the most compact
    /// representation.
    fn with_struct_tuple(self) -> StructTupleSerializer<Self> {
        StructTupleSerializer::new(self)
    }
}

impl<W: Write> Ext for Serializer<W> {}

impl<W: Write> UnderlyingWrite for Serializer<W> {
    type Write = W;

    fn get_ref(&self) -> &Self::Write {
        &self.wr
    }

    fn get_mut(&mut self) -> &mut Self::Write {
        &mut self.wr
    }

    fn into_inner(self) -> Self::Write {
        self.wr
    }
}

/// Part of serde serialization API.
#[derive(Debug)]
pub struct Compound<'a, W: 'a> {
    se: &'a mut Serializer<W>,
}

impl<'a, W: Write + 'a> SerializeSeq for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write + 'a> SerializeTuple for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write + 'a> SerializeTupleStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write + 'a> SerializeTupleVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write + 'a> SerializeMap for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        key.serialize(&mut *self.se)
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write + 'a> SerializeStruct for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, value: &T) ->
        Result<(), Self::Error>
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write + 'a> SerializeStructVariant for Compound<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, value: &T) ->
        Result<(), Self::Error>
    {
        value.serialize(&mut *self.se)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W> serde::Serializer for &'a mut Serializer<W>
where
    W: Write
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a, W>;
    type SerializeTuple = Compound<'a, W>;
    type SerializeTupleStruct = Compound<'a, W>;
    type SerializeTupleVariant = Compound<'a, W>;
    type SerializeMap = Compound<'a, W>;
    type SerializeStruct = Compound<'a, W>;
    type SerializeStructVariant = Compound<'a, W>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        encode::write_bool(&mut self.wr, v)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(err)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        encode::write_sint(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        encode::write_uint(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        encode::write_f32(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        encode::write_f64(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        encode::write_str(&mut self.wr, v)?;
        Ok(())
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Self::Ok, Self::Error> {
        encode::write_bin_len(&mut self.wr, value.len() as u32)?;
        self.wr
            .write_all(value)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidDataWrite(err)))
    }

    fn serialize_none(self) -> Result<(), Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized + serde::Serialize>(self, v: &T) -> Result<(), Self::Error> {
        v.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        encode::write_nil(&mut self.wr)
            .map_err(|err| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(err)))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        encode::write_array_len(&mut self.wr, 0)?;
        Ok(())
    }

    fn serialize_unit_variant(self, _name: &str, idx: u32, _variant: &str) ->
        Result<Self::Ok, Self::Error>
    {
        // encode as a map from variant idx to nil, like: {idx => nil}
        encode::write_map_len(&mut self.wr, 1)?;
        self.serialize_u32(idx)?;
        encode::write_nil(&mut self.wr).map_err(|e| Error::InvalidValueWrite(ValueWriteError::InvalidMarkerWrite(e)))?;
        Ok(())
    }

    fn serialize_newtype_struct<T: ?Sized + serde::Serialize>(self, _name: &'static str, value: &T) -> Result<(), Self::Error> {
        // Encode as if it's inner type.
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + serde::Serialize>(self, _name: &'static str, idx: u32, _variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error> {
        // encode as a map from variant idx to its attributed data, like: {idx => value}
        encode::write_map_len(&mut self.wr, 1)?;
        self.serialize_u32(idx)?;
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Error> {
        let len = match len {
            Some(len) => len,
            None => return Err(Error::UnknownLength),
        };

        encode::write_array_len(&mut self.wr, len as u32)?;

        self.compound()
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(self, _name: &'static str, len: usize) ->
        Result<Self::SerializeTupleStruct, Self::Error>
    {
        self.serialize_tuple(len)
    }

    fn serialize_tuple_variant(self,  name: &'static str,  idx: u32,  _variant: &'static str,  len: usize) ->
        Result<Self::SerializeTupleVariant, Error>
    {
        // encode as a map from variant idx to a sequence of its attributed data, like: {idx => [v1,...,vN]}
        encode::write_map_len(&mut self.wr, 1)?;
        self.serialize_u32(idx)?;
        self.serialize_tuple_struct(name, len)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Error> {
        match len {
            Some(len) => {
                encode::write_map_len(&mut self.wr, len as u32)?;
                self.compound()
            }
            None => Err(Error::UnknownLength),
        }
    }

    fn serialize_struct(self, _name: &'static str, len: usize) ->
        Result<Self::SerializeStruct, Self::Error>
    {
        encode::write_array_len(&mut self.wr, len as u32)?;
        self.compound()
    }

    fn serialize_struct_variant(self, name: &'static str, id: u32, _variant: &'static str, len: usize) ->
        Result<Self::SerializeStructVariant, Error>
    {
        // encode as a map from variant idx to a sequence of its attributed data, like: {idx => [v1,...,vN]}
        encode::write_map_len(&mut self.wr, 1)?;
        self.serialize_u32(id)?;
        self.serialize_struct(name, len)
    }
}

/// Serialize the given data structure as MessagePack into the I/O stream.
/// This function uses compact representation - structures as arrays
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn write<W, T>(wr: &mut W, val: &T) -> Result<(), Error>
where
    W: Write + ?Sized,
    T: Serialize + ?Sized
{
    val.serialize(&mut Serializer::new(wr))
}

/// Serialize the given data structure as MessagePack into the I/O stream.
/// This function serializes structures as maps
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn write_named<W, T>(wr: &mut W, val: &T) -> Result<(), Error>
where
    W: Write + ?Sized,
    T: Serialize + ?Sized
{
    let mut se = Serializer::new(wr)
        .with_struct_map();
    val.serialize(&mut se)
}

/// Serialize the given data structure as a MessagePack byte vector.
/// This method uses compact representation, structs are serialized as arrays
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn to_vec<T>(val: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized
{
    let mut wr = Vec::with_capacity(128);
    write(&mut wr, val)?;
    Ok(wr)
}

/// Serializes data structure into byte vector as a map
/// Resulting MessagePack message will contain field names
///
/// # Errors
///
/// Serialization can fail if `T`'s implementation of `Serialize` decides to fail.
#[inline]
pub fn to_vec_named<T>(val: &T) -> Result<Vec<u8>, Error>
where
    T: Serialize + ?Sized
{
    let mut wr = Vec::with_capacity(128);
    write_named(&mut wr, val)?;
    Ok(wr)
}
