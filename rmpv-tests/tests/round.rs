#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_bytes::ByteBuf;

use rmpv::Value;

/// Tests that the following round-trip conditions are met:
/// - `T`     -> `[u8]`  == `Value` -> `[u8]`.
/// - `T`     -> `Value` == `Value`.
/// - `[u8]`  -> `T`     == `T`.
/// - `[u8]`  -> `Value` == `Value`.
/// - `Value` -> `T`     == `T`.
fn test_round<'de, T>(var: T, val: Value)
    where T: Debug + PartialEq + Serialize + DeserializeOwned
{
    // Serialize part.
    // Test that `T` -> `[u8]` equals with serialization from `Value` -> `[u8]`.
    let buf_from_var = rmps::to_vec(&var).unwrap();
    let buf_from_val = rmps::to_vec(&val).unwrap();
    assert_eq!(buf_from_var, buf_from_val);

    // Test that `T` -> `Value` equals with the given `Value`.
    let val_from_var = rmpv::ext::to_value(&var).unwrap();
    assert_eq!(val, val_from_var);

    // Deserialize part.
    // Test that `[u8]` -> `T` equals with the given `T`.
    let var_from_buf: T = rmps::from_slice(buf_from_var.as_slice()).unwrap();
    assert_eq!(var, var_from_buf);

    // Test that `[u8]` -> `Value` equals with the given `Value`.
    let val_from_buf: Value = rmps::from_slice(buf_from_var.as_slice()).unwrap();
    assert_eq!(val, val_from_buf);

    // Test that `Value` -> `T` equals with the given `T`.
    let var_from_val: T = rmpv::ext::from_value(val_from_buf).unwrap();
    assert_eq!(var, var_from_val);
}

#[test]
fn pass_nil() {
    test_round((), Value::Nil);
}

#[test]
fn pass_bool() {
    test_round(true, Value::Boolean(true));
    test_round(false, Value::Boolean(false));
}

#[test]
fn pass_uint() {
    test_round(u8::min_value(), Value::from(u8::min_value()));
    test_round(u8::max_value(), Value::from(u8::max_value()));
    test_round(u16::max_value(), Value::from(u16::max_value()));
    test_round(u32::max_value(), Value::from(u32::max_value()));
    test_round(u64::max_value(), Value::from(u64::max_value()));
}

#[test]
fn pass_sint() {
    test_round(i8::min_value(), Value::from(i8::min_value()));
    test_round(i8::max_value(), Value::from(i8::max_value()));
    test_round(i16::min_value(), Value::from(i16::min_value()));
    test_round(i16::max_value(), Value::from(i16::max_value()));
    test_round(i32::min_value(), Value::from(i32::min_value()));
    test_round(i32::max_value(), Value::from(i32::max_value()));
    test_round(i64::min_value(), Value::from(i64::min_value()));
    test_round(i64::max_value(), Value::from(i64::max_value()));
}

#[test]
fn pass_f32() {
    test_round(std::f32::MAX, Value::from(std::f32::MAX));
}

#[test]
fn pass_f64() {
    test_round(0.00, Value::from(0.00));
    test_round(42.0, Value::from(42.0));
}

#[test]
fn pass_str() {
    test_round("John".to_string(), Value::from("John"));
}

#[test]
fn pass_bin() {
    test_round(ByteBuf::from(&[0xcc, 0x80][..]), Value::from(&[0xcc, 0x80][..]));
}

#[test]
fn pass_vec() {
    test_round([0, 42], Value::from(vec![Value::from(0), Value::from(42)]));
    test_round(vec![0, 42], Value::from(vec![Value::from(0), Value::from(42)]));
}

#[test]
fn pass_newtype_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Newtype(String);

    test_round(Newtype("John".into()), Value::from("John"));
}

#[test]
fn pass_ext_struct() {
    #[derive(Debug, PartialEq)]
    enum ExtStruct {
        One(u8),
        Two(u8)
    }

    struct ExtStructVisitor;

    use serde::de::Unexpected;
    use serde_bytes::ByteBuf;

    impl Serialize for ExtStruct {
        fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where S: serde::ser::Serializer
        {
            let value = match self {
                ExtStruct::One(data) => {
                    let tag = 1 as i8;
                    let byte_buf = ByteBuf::from(vec![*data]);

                    (tag, byte_buf)
                }
                ExtStruct::Two(data) => {
                    let tag = 2 as i8;
                    let byte_buf = ByteBuf::from(vec![*data]);

                    (tag, byte_buf)
                }
            };
            s.serialize_newtype_struct(rmps::MSGPACK_EXT_STRUCT_NAME, &value)
        }
    }

    impl<'de> serde::de::Visitor<'de> for ExtStructVisitor {
        type Value = ExtStruct;

        fn expecting(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(fmt, "a sequence of tag & binary")
        }

        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_tuple(2, self)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where A: serde::de::SeqAccess<'de>
        {
            let tag: i8 = seq.next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
            let data: ByteBuf = seq.next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

            if tag == 1 {
                Ok(ExtStruct::One(data[0]))
            } else if tag == 2 {
                Ok(ExtStruct::Two(data[0]))
            } else {
                let unexp = Unexpected::Signed(tag as i64);
                Err(serde::de::Error::invalid_value(unexp, &self))
            }
        }
    }

    impl<'de> serde::de::Deserialize<'de> for ExtStruct {
        fn deserialize<D>(deserializer: D) -> Result<ExtStruct, D::Error>
            where D: serde::Deserializer<'de>,
        {
            let visitor = ExtStructVisitor;
            deserializer.deserialize_newtype_struct(rmps::MSGPACK_EXT_STRUCT_NAME, visitor)
        }
    }

    test_round(ExtStruct::One(5), Value::Ext(1, vec![5]));
}

#[test]
fn pass_derive_serde_ext_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    #[serde(rename = "_ExtStruct")]
    struct ExtStruct((i8, serde_bytes::ByteBuf));

    test_round(ExtStruct((2, serde_bytes::ByteBuf::from(vec![5]))),
               Value::Ext(2, vec![5]));
}
