//! Extend MessagePack serialization using wrappers.

use std::io::Write;

use rmp::encode;
use serde::{Serialize, Serializer};
use serde::ser::{SerializeStruct, SerializeStructVariant};

use encode::{Error, Ext, UnderlyingWrite};

/// Serializer wrapper, that overrides struct serialization by packing as a map with field names.
///
/// MessagePack specification does not tell how to serialize structs. This trait allows you to
/// extend serialization to match your app's requirements.
///
/// Default `Serializer` implementation writes structs as a tuple, i.e. only its length is encoded,
/// because it is the most compact representation.
#[derive(Debug)]
pub struct StructMapSerializer<S> {
    se: S,
}

impl<S> StructMapSerializer<S> {
    /// Wraps a serializer overriding its struct serialization methods to be able to serialize
    /// structs as a map with field names.
    pub fn new(se: S) -> Self {
        Self { se: se }
    }
}

impl<S: UnderlyingWrite> Ext for StructMapSerializer<S> {}

impl<S, W> UnderlyingWrite for StructMapSerializer<S>
where
    S: UnderlyingWrite<Write = W>,
    W: Write
{
    type Write = W;

    fn get_ref(&self) -> &Self::Write {
        self.se.get_ref()
    }

    fn get_mut(&mut self) -> &mut Self::Write {
        self.se.get_mut()
    }

    fn into_inner(self) -> Self::Write {
        self.se.into_inner()
    }
}

impl<'a, S> Serializer for &'a mut StructMapSerializer<S>
where
    S: UnderlyingWrite,
    for<'b> &'b mut S: Serializer<Ok = (), Error = Error>
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = <&'a mut S as Serializer>::SerializeSeq;
    type SerializeTuple = <&'a mut S as Serializer>::SerializeTuple;
    type SerializeTupleStruct = <&'a mut S as Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant = <&'a mut S as Serializer>::SerializeTupleVariant;
    type SerializeMap = <&'a mut S as Serializer>::SerializeMap;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_bool(v)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i8(v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i16(v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i32(v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i64(v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u8(v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u16(v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u32(v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u64(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_f32(v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_f64(v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_char(v)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_str(v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_bytes(v)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_none()
    }

    #[inline]
    fn serialize_some<T: ? Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        self.se.serialize_some(value)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_unit()
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_unit_struct(name)
    }

    #[inline]
    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_unit_variant(name, variant_index, variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ? Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        self.se.serialize_newtype_struct(name, value)
    }

    #[inline]
    fn serialize_newtype_variant<T: ? Sized>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize
    {
        self.se.serialize_newtype_variant(name, variant_index, variant, value)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.se.serialize_seq(len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.se.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.se.serialize_tuple_struct(name, len)
    }

    #[inline]
    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.se.serialize_tuple_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.se.serialize_map(len)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        encode::write_map_len(self.se.get_mut(), len as u32)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        encode::write_array_len(self.se.get_mut(), 2)?;
        self.se.serialize_u32(variant_index)?;
        encode::write_map_len(self.se.get_mut(), len as u32)?;
        Ok(self)
    }
}

impl<'a, S> SerializeStruct for &'a mut StructMapSerializer<S>
where
    S: UnderlyingWrite,
    for<'b> &'b mut S: Serializer<Ok = (), Error = Error>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) ->
        Result<(), Self::Error>
    {
        encode::write_str(self.se.get_mut(), key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S> SerializeStructVariant for &'a mut StructMapSerializer<S>
where
    S: UnderlyingWrite,
    for<'b> &'b mut S: Serializer<Ok = (), Error = Error>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, key: &'static str, value: &T) ->
        Result<(), Self::Error>
    {
        encode::write_str(self.se.get_mut(), key)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

/// Serializer wrapper, that overrides struct serialization by packing as a tuple without field
/// names.
#[derive(Debug)]
pub struct StructTupleSerializer<S> {
    se: S,
}

impl<'a, S> StructTupleSerializer<S>
where
    S: UnderlyingWrite
{
    /// Wraps a serializer overriding its struct serialization methods to be able to serialize
    /// structs as an array without field names.
    pub fn new(se: S) -> Self {
        Self { se: se }
    }
}

impl<S: UnderlyingWrite> Ext for StructTupleSerializer<S> {}

impl<S, W> UnderlyingWrite for StructTupleSerializer<S>
where
    S: UnderlyingWrite<Write = W>,
    W: Write
{
    type Write = W;

    fn get_ref(&self) -> &Self::Write {
        self.se.get_ref()
    }

    fn get_mut(&mut self) -> &mut Self::Write {
        self.se.get_mut()
    }

    fn into_inner(self) -> Self::Write {
        self.se.into_inner()
    }
}

impl<'a, S> Serializer for &'a mut StructTupleSerializer<S>
where
    S: UnderlyingWrite,
    for<'b> &'b mut S: Serializer<Ok = (), Error = Error>
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = <&'a mut S as Serializer>::SerializeSeq;
    type SerializeTuple = <&'a mut S as Serializer>::SerializeTuple;
    type SerializeTupleStruct = <&'a mut S as Serializer>::SerializeTupleStruct;
    type SerializeTupleVariant = <&'a mut S as Serializer>::SerializeTupleVariant;
    type SerializeMap = <&'a mut S as Serializer>::SerializeMap;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    #[inline]
    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_bool(v)
    }

    #[inline]
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i8(v)
    }

    #[inline]
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i16(v)
    }

    #[inline]
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i32(v)
    }

    #[inline]
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_i64(v)
    }

    #[inline]
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u8(v)
    }

    #[inline]
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u16(v)
    }

    #[inline]
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u32(v)
    }

    #[inline]
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_u64(v)
    }

    #[inline]
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_f32(v)
    }

    #[inline]
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_f64(v)
    }

    #[inline]
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_char(v)
    }

    #[inline]
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_str(v)
    }

    #[inline]
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_bytes(v)
    }

    #[inline]
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_none()
    }

    #[inline]
    fn serialize_some<T: ? Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize
    {
        self.se.serialize_some(value)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_unit()
    }

    #[inline]
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_unit_struct(name)
    }

    #[inline]
    fn serialize_unit_variant(self, name: &'static str, variant_index: u32, variant: &'static str) -> Result<Self::Ok, Self::Error> {
        self.se.serialize_unit_variant(name, variant_index, variant)
    }

    #[inline]
    fn serialize_newtype_struct<T: ? Sized>(self, name: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize
    {
        self.se.serialize_newtype_struct(name, value)
    }

    #[inline]
    fn serialize_newtype_variant<T: ? Sized>(self, name: &'static str, variant_index: u32, variant: &'static str, value: &T) -> Result<Self::Ok, Self::Error>
        where
            T: Serialize
    {
        self.se.serialize_newtype_variant(name, variant_index, variant, value)
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.se.serialize_seq(len)
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.se.serialize_tuple(len)
    }

    #[inline]
    fn serialize_tuple_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.se.serialize_tuple_struct(name, len)
    }

    #[inline]
    fn serialize_tuple_variant(self, name: &'static str, variant_index: u32, variant: &'static str, len: usize) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.se.serialize_tuple_variant(name, variant_index, variant, len)
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.se.serialize_map(len)
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct, Self::Error> {
        encode::write_array_len(self.se.get_mut(), len as u32)?;
        Ok(self)
    }

    #[inline]
    fn serialize_struct_variant(self, _name: &'static str, variant_index: u32, _variant: &'static str, len: usize) -> Result<Self::SerializeStructVariant, Self::Error> {
        encode::write_array_len(&mut self.se.get_mut(), 2)?;
        self.se.serialize_u32(variant_index)?;
        encode::write_array_len(self.se.get_mut(), len as u32)?;
        Ok(self)
    }
}

impl<'a, S> SerializeStruct for &'a mut StructTupleSerializer<S>
where
    S: UnderlyingWrite,
    for<'b> &'b mut S: Serializer<Ok = (), Error = Error>,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, value: &T) ->
        Result<(), Self::Error>
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, S> SerializeStructVariant for &'a mut StructTupleSerializer<S>
where
    S: UnderlyingWrite,
    for<'b> &'b mut S: Serializer<Ok = (), Error = Error>
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, _key: &'static str, value: &T) ->
        Result<(), Self::Error>
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
