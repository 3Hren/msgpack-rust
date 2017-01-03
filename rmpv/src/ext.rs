use serde::{self, Serialize, Deserialize};
use serde::bytes::Bytes;

use Value;

impl Serialize for Value {
    #[inline]
    fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
        where S: serde::Serializer
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
                let mut state = try!(s.serialize_seq(Some(array.len())));
                for item in array {
                    try!(s.serialize_seq_elt(&mut state, item));
                }
                s.serialize_seq_end(state)
            }
            Value::Map(ref map) => {
                let mut state = try!(s.serialize_map(Some(map.len())));
                for &(ref key, ref val) in map {
                    try!(s.serialize_map_key(&mut state, key));
                    try!(s.serialize_map_value(&mut state, val));
                }
                s.serialize_map_end(state)
            }
            Value::Ext(ty, ref buf) => {
                try!(s.serialize_i8(ty));
                buf.serialize(s)
            }
        }
    }
}

impl Deserialize for Value {
    #[inline]
    fn deserialize<D>(de: &mut D) -> Result<Self, D::Error>
        where D: serde::Deserializer
    {
        struct ValueVisitor;

        impl serde::de::Visitor for ValueVisitor {
            type Value = Value;

            #[inline]
            fn visit_some<D>(&mut self, de: &mut D) -> Result<Value, D::Error>
                where D: serde::de::Deserializer
            {
                Deserialize::deserialize(de)
            }

            #[inline]
            fn visit_none<E>(&mut self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_unit<E>(&mut self) -> Result<Value, E> {
                Ok(Value::Nil)
            }

            #[inline]
            fn visit_bool<E>(&mut self, value: bool) -> Result<Value, E> {
                Ok(Value::Boolean(value))
            }

            #[inline]
            fn visit_u64<E>(&mut self, value: u64) -> Result<Value, E> {
                Ok(Value::U64(value))
            }

            #[inline]
            fn visit_i64<E>(&mut self, value: i64) -> Result<Value, E> {
                if value < 0 {
                    Ok(Value::I64(value))
                } else {
                    Ok(Value::U64(value as u64))
                }
            }

            #[inline]
            fn visit_f32<E>(&mut self, value: f32) -> Result<Value, E> {
                Ok(Value::F32(value))
            }

            #[inline]
            fn visit_f64<E>(&mut self, value: f64) -> Result<Value, E> {
                Ok(Value::F64(value))
            }

            #[inline]
            fn visit_string<E>(&mut self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
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
                let values = values.into_iter().collect();

                Ok(Value::Array(values))
            }

            #[inline]
            fn visit_bytes<E>(&mut self, v: &[u8]) -> Result<Self::Value, E>
                where E: serde::de::Error
            {
                Ok(Value::Binary(v.to_owned()))
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
