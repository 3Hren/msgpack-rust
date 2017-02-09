use std::fmt::{self, Formatter};
use serde::{self, Serialize, Serializer, Deserialize, Deserializer};
use serde::bytes::Bytes;
use serde::ser::{SerializeSeq, SerializeMap};

use Value;

impl Serialize for Value {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        match *self {
            Value::Nil => s.serialize_unit(),
            Value::Boolean(v) => s.serialize_bool(v),
            Value::I64(v) => s.serialize_i64(v),
            Value::U64(v) => s.serialize_u64(v),
            Value::F32(v) => s.serialize_f32(v),
            Value::F64(v) => s.serialize_f64(v),
            Value::String(ref v) => s.serialize_str(v),
            Value::Binary(ref v) => Bytes::from(v).serialize(s),
            Value::Array(ref array) => {
                let mut state = s.serialize_seq(Some(array.len()))?;
                for item in array {
                    state.serialize_element(item)?;
                }
                state.end()
            }
            Value::Map(ref map) => {
                let mut state = s.serialize_map(Some(map.len()))?;
                for &(ref key, ref val) in map {
                    state.serialize_entry(key, val)?;
                }
                state.end()
            }
            Value::Ext(ty, ref buf) => {
                let mut state = s.serialize_seq(Some(2))?;
                state.serialize_element(&ty)?;
                state.serialize_element(buf)?;
                state.end()
            }
        }
    }
}

impl Deserialize for Value {
    #[inline]
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
        where D: Deserializer
    {
        struct ValueVisitor;

        impl serde::de::Visitor for ValueVisitor {
            type Value = Value;

            fn expecting(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
                fmt.write_str("any valid MessagePack value")
            }

            #[inline]
            fn visit_some<D>(self, de: D) -> Result<Value, D::Error>
                where D: Deserializer
            {
                Deserialize::deserialize(de)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Boolean(value))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::U64(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                if value < 0 {
                    Ok(Value::I64(value))
                } else {
                    Ok(Value::U64(value as u64))
                }
            }

            #[inline]
            fn visit_f32<E>(self, value: f32) -> Result<Value, E> {
                Ok(Value::F32(value))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::F64(value))
            }

            #[inline]
            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<Value, E>
                where E: serde::de::Error
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_seq<V>(self, visitor: V) -> Result<Value, V::Error>
                where V: serde::de::SeqVisitor
            {
                let values: Vec<Value> = try!(serde::de::impls::VecVisitor::new()
                    .visit_seq(visitor));
                let values = values.into_iter().collect();

                Ok(Value::Array(values))
            }

            #[inline]
            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Binary(v.to_owned()))
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut pairs = vec![];

                loop {
                    let key: Option<Value> = try!(visitor.visit_key());
                    if let Some(key) = key {
                        let value: Value = try!(visitor.visit_value());

                        pairs.push((key, value));
                    } else {
                        break;
                    }
                }

                Ok(Value::Map(pairs))
            }
        }

        de.deserialize(ValueVisitor)
    }
}
