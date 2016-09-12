extern crate rmp;
extern crate serde;

use serde::bytes::Bytes;
use serde::ser::{Serialize, SeqVisitor, MapVisitor};
use serde::de::Deserialize;

use rmp::value::{Float, Integer};

pub use decode::Deserializer;
pub use encode::Serializer;

pub mod decode;
pub mod encode;

/// Owning wrapper over rmp `Value` to allow serialization and deserialization.
#[derive(Debug, PartialEq, Clone)]
pub struct Value(pub rmp::Value);

/// Non-owning wrapper over rmp `Value` reference to allow serialization and deserialization.
pub struct BorrowedValue<'a>(pub &'a rmp::Value);

impl<T: Into<rmp::Value>> From<T> for Value {
    fn from(val: T) -> Value {
        Value(val.into())
    }
}

impl<'a> Serialize for BorrowedValue<'a> {
    #[inline]
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::ser::Serializer
    {
        match *self.0 {
            rmp::Value::Nil => s.serialize_unit(),
            rmp::Value::Boolean(v) => s.serialize_bool(v),
            rmp::Value::Integer(Integer::I64(v)) => s.serialize_i64(v),
            rmp::Value::Integer(Integer::U64(v)) => s.serialize_u64(v),
            rmp::Value::Float(Float::F32(v)) => s.serialize_f32(v),
            rmp::Value::Float(Float::F64(v)) => s.serialize_f64(v),
            rmp::Value::String(ref v) => s.serialize_str(v),
            rmp::Value::Binary(ref v) => Bytes::from(v).serialize(s),
            rmp::Value::Array(ref array) => {
                struct Visitor<'a> {
                    array: &'a [rmp::Value],
                }

                impl<'a> SeqVisitor for Visitor<'a> {
                    fn visit<S>(&mut self, _s: &mut S) -> Result<Option<()>, S::Error>
                        where S: serde::ser::Serializer
                    {
                        Ok(None)
                    }

                    fn len(&self) -> Option<usize> {
                        Some(self.array.len())
                    }
                }

                // TODO: let state = try!(s.serialize_seq(Some(array.len()))); when serde 0.8 comes.
                try!(s.serialize_seq(Visitor { array: &array[..] }));
                for elt in array {
                    // TODO: try!(s.serialize_seq_elt(&mut state, elt)); when serde 0.8 comes.
                    try!(s.serialize_seq_elt(BorrowedValue(elt)));
                }
                // TODO: try!(s.serialize_seq_end(&mut state)) when serde 0.8 comes.
                Ok(())
            }
            rmp::Value::Map(ref map) => {
                struct Visitor<'a> {
                    map: &'a [(rmp::Value, rmp::Value)],
                }

                impl<'a> MapVisitor for Visitor<'a> {
                    fn visit<S>(&mut self, _s: &mut S) -> Result<Option<()>, S::Error>
                        where S: serde::ser::Serializer
                    {
                        Ok(None)
                    }

                    fn len(&self) -> Option<usize> {
                        Some(self.map.len())
                    }
                }

                try!(s.serialize_map(Visitor { map: &map[..] }));
                for &(ref key, ref val) in map {
                    try!(s.serialize_map_elt(BorrowedValue(key), BorrowedValue(val)));
                }
                Ok(())
            }
            rmp::Value::Ext(ty, ref buf) => {
                try!(s.serialize_i8(ty));
                buf.serialize(s)
            }
        }
    }
}


impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::ser::Serializer
    {
        BorrowedValue(&self.0).serialize(s)
    }
}

impl Deserialize for Value {
    #[inline]
    fn deserialize<D>(deserializer: &mut D) -> Result<Self, D::Error>
        where D: serde::de::Deserializer
    {
        struct ValueVisitor;

        impl serde::de::Visitor for ValueVisitor {
            type Value = Value;

            #[inline]
            fn visit_some<D>(&mut self, deserializer: &mut D) -> Result<Value, D::Error>
                where D: serde::de::Deserializer
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_none<E>(&mut self) -> Result<Value, E> {
                Ok(Value(rmp::Value::Nil))
            }

            #[inline]
            fn visit_unit<E>(&mut self) -> Result<Value, E> {
                Ok(Value(rmp::Value::Nil))
            }

            #[inline]
            fn visit_bool<E>(&mut self, value: bool) -> Result<Value, E> {
                Ok(Value(rmp::Value::Boolean(value)))
            }

            #[inline]
            fn visit_u64<E>(&mut self, value: u64) -> Result<Value, E> {
                Ok(Value(rmp::Value::Integer(rmp::value::Integer::U64(value))))
            }

            #[inline]
            fn visit_i64<E>(&mut self, value: i64) -> Result<Value, E> {
                if value < 0 {
                    Ok(Value(rmp::Value::Integer(rmp::value::Integer::I64(value))))
                } else {
                    Ok(Value(rmp::Value::Integer(rmp::value::Integer::U64(value as u64))))
                }
            }

            #[inline]
            fn visit_f32<E>(&mut self, value: f32) -> Result<Value, E> {
                Ok(Value(rmp::Value::Float(rmp::value::Float::F32(value))))
            }

            #[inline]
            fn visit_f64<E>(&mut self, value: f64) -> Result<Value, E> {
                Ok(Value(rmp::Value::Float(rmp::value::Float::F64(value))))
            }

            #[inline]
            fn visit_string<E>(&mut self, value: String) -> Result<Value, E> {
                Ok(Value(rmp::Value::String(value)))
            }

            #[inline]
            fn visit_str<E>(&mut self, value: &str) -> Result<Value, E>
                where E: serde::de::Error
            {
                self.visit_string(String::from(value))
            }

            #[inline]
            fn visit_seq<V>(&mut self, visitor: V) -> Result<Value, V::Error>
                where V: serde::de::SeqVisitor
            {
                let values: Vec<Value> = try!(serde::de::impls::VecVisitor::new()
                    .visit_seq(visitor));
                let values = values.into_iter().map(|v| v.0).collect();

                Ok(Value(rmp::Value::Array(values)))
            }

            #[inline]
            fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(Value(rmp::Value::Binary(v.to_owned())))
            }

            #[inline]
            fn visit_map<V>(&mut self, mut visitor: V) -> Result<Value, V::Error>
                where V: serde::de::MapVisitor
            {
                let mut pairs = vec![];

                loop {
                    let key: Option<Value> = try!(visitor.visit_key());
                    if let Some(key) = key {
                        let value: Value = try!(visitor.visit_value());

                        pairs.push((key.0, value.0));
                    } else {
                        break;
                    }
                }

                Ok(Value(rmp::Value::Map(pairs)))
            }
        }

        deserializer.deserialize(ValueVisitor)
    }
}
