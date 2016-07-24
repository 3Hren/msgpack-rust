extern crate rmp;
extern crate serde;

use serde::bytes::Bytes;
use serde::ser::{Serialize, SeqVisitor, MapVisitor};

use rmp::value::{Float, Integer};

pub use decode::Deserializer;
pub use encode::Serializer;

pub mod decode;
pub mod encode;

/// Owning wrapper over rmp `Value` to allow serialization and deserialization.
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
