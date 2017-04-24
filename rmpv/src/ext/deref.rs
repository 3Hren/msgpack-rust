use std::iter::ExactSizeIterator;
use std::slice::Iter;

use serde::Deserializer;
use serde::de::{self, DeserializeSeed, IntoDeserializer, Unexpected, Visitor};

use {Integer, IntPriv, ValueRef};

use super::{deserialize_from, Error, SeqDeserializer, ValueExt};

impl<'de> Deserializer<'de> for &'de ValueRef<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        match *self {
            ValueRef::Nil => visitor.visit_unit(),
            ValueRef::Boolean(v) => visitor.visit_bool(v),
            ValueRef::Integer(Integer { n }) => {
                match n {
                    IntPriv::PosInt(v) => visitor.visit_u64(v),
                    IntPriv::NegInt(v) => visitor.visit_i64(v)
                }
            }
            ValueRef::F32(v) => visitor.visit_f32(v),
            ValueRef::F64(v) => visitor.visit_f64(v),
            ValueRef::String(v) => {
                match v.s {
                    Ok(v) => visitor.visit_borrowed_str(v),
                    Err(v) => visitor.visit_borrowed_bytes(v.0),
                }
            }
            ValueRef::Binary(v) => visitor.visit_borrowed_bytes(v),
            ValueRef::Array(ref v) => {
                let len = v.len();
                let mut de = SeqDeserializer::new(v.into_iter());
                let seq = visitor.visit_seq(&mut de)?;
                if de.iter.len() == 0 {
                    Ok(seq)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in array"))
                }
            }
            ValueRef::Map(ref v) => {
                let len = v.len();
                let mut de = MapRefDeserializer::new(v.into_iter());
                let map = visitor.visit_map(&mut de)?;
                if de.iter.len() == 0 {
                    Ok(map)
                } else {
                    Err(de::Error::invalid_length(len, &"fewer elements in map"))
                }
            }
            ValueRef::Ext(..) => {
                unimplemented!();
            }
        }
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        if let &ValueRef::Nil = self {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn deserialize_enum<V>(self, _name: &str, _variants: &'static [&'static str], visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        match self {
            &ValueRef::Array(ref v) => {
                let len = v.len();
                let mut iter = v.into_iter();
                if !(len == 1 || len == 2) {
                    return Err(de::Error::invalid_length(len, &"array with one or two elements"));
                }

                let id = match iter.next() {
                    Some(id) => deserialize_from(id)?,
                    None => {
                        return Err(de::Error::invalid_length(len, &"array with one or two elements"));
                    }
                };

                visitor.visit_enum(EnumRefDeserializer::new(id, iter.next()))
            }
            other => Err(de::Error::invalid_type(other.unexpected(), &"array, map or int")),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        match self {
            &ValueRef::Array(ref v) => {
                let iter = v.into_iter();
                if iter.len() != 1 {
                    return Err(de::Error::invalid_length(iter.len(), &"array with one element"));
                }

                visitor.visit_seq(SeqDeserializer::new(iter))
            }
            other => Err(de::Error::invalid_type(other.unexpected(), &"array")),
        }
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        match self {
            &ValueRef::Array(ref v) => {
                if v.len() == 0 {
                    visitor.visit_unit()
                } else {
                    Err(de::Error::invalid_length(v.len(), &"empty array"))
                }
            }
            other => Err(de::Error::invalid_type(other.unexpected(), &"empty array")),
        }
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit seq
        bytes byte_buf map tuple_struct struct
        identifier tuple ignored_any
    }
}

pub struct MapRefDeserializer<'de> {
    val: Option<&'de ValueRef<'de>>,
    iter: Iter<'de, (ValueRef<'de>, ValueRef<'de>)>,
}

impl<'de> MapRefDeserializer<'de> {
    fn new(iter: Iter<'de, (ValueRef<'de>, ValueRef<'de>)>) -> Self {
        Self {
            val: None,
            iter: iter,
        }
    }
}

impl<'de> de::MapAccess<'de> for MapRefDeserializer<'de> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
        where T: DeserializeSeed<'de>
    {
        match self.iter.next() {
            Some(&(ref key, ref val)) => {
                self.val = Some(val);
                seed.deserialize(key).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
        where T: DeserializeSeed<'de>
    {
        match self.val.take() {
            Some(val) => seed.deserialize(val),
            None => Err(de::Error::custom("value is missing")),
        }
    }
}

impl<'de> Deserializer<'de> for MapRefDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where V: Visitor<'de>
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool u8 u16 u32 u64 i8 i16 i32 i64 f32 f64 char str string unit option
        seq bytes byte_buf map unit_struct newtype_struct
        tuple_struct struct identifier tuple enum ignored_any
    }
}

pub struct EnumRefDeserializer<'de> {
    id: u32,
    value: Option<&'de ValueRef<'de>>,
}

impl<'de> EnumRefDeserializer<'de> {
    pub fn new(id: u32, value: Option<&'de ValueRef<'de>>) -> Self {
        Self {
            id: id,
            value: value,
        }
    }
}

impl<'de> de::EnumAccess<'de> for EnumRefDeserializer<'de> {
    type Error = Error;
    type Variant = VariantRefDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
        where V: de::DeserializeSeed<'de>
    {
        let variant = self.id.into_deserializer();
        let visitor = VariantRefDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

pub struct VariantRefDeserializer<'de> {
    value: Option<&'de ValueRef<'de>>,
}

impl<'de> de::VariantAccess<'de> for VariantRefDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        // Can accept only [u32].
        match self.value {
            Some(&ValueRef::Array(ref v)) => {
                if v.len() == 0 {
                    Ok(())
                } else {
                    Err(de::Error::invalid_value(Unexpected::Seq, &"empty array"))
                }
            }
            Some(v) => Err(de::Error::invalid_value(v.unexpected(), &"empty array")),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
        where T: de::DeserializeSeed<'de>
    {
        // Can accept both [u32, T...] and [u32, [T]] cases.
        match self.value {
            Some(&ValueRef::Array(ref v)) => {
                let len = v.len();
                let mut iter = v.into_iter();
                if len > 1 {
                    seed.deserialize(SeqDeserializer::new(iter))
                } else {
                    let val = match iter.next() {
                        Some(val) => seed.deserialize(val),
                        None => return Err(de::Error::invalid_length(len, &"array with one element")),
                    };

                    if iter.next().is_some() {
                        Err(de::Error::invalid_length(len, &"array with one element"))
                    } else {
                        val
                    }
                }
            }
            Some(v) => seed.deserialize(v),
            None => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"newtype variant")),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>
    {
        // Can accept [u32, [T...]].
        match self.value {
            Some(&ValueRef::Array(ref v)) => {
                Deserializer::deserialize_any(SeqDeserializer::new(v.into_iter()), visitor)
            }
            Some(v) => Err(de::Error::invalid_type(v.unexpected(), &"tuple variant")),
            None => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"tuple variant"))
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value, Error>
        where V: Visitor<'de>,
    {
        match self.value {
            Some(&ValueRef::Array(ref v)) => {
                Deserializer::deserialize_any(SeqDeserializer::new(v.into_iter()), visitor)
            }
            Some(&ValueRef::Map(ref v)) => {
                Deserializer::deserialize_any(MapRefDeserializer::new(v.into_iter()), visitor)
            }
            Some(v) => Err(de::Error::invalid_type(v.unexpected(), &"struct variant")),
            None => Err(de::Error::invalid_type(Unexpected::UnitVariant, &"struct variant"))
        }
    }
}
