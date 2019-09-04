#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use std::borrow::Cow;
use std::collections::BTreeMap;

use serde::Serialize;
use serde_bytes::{Bytes, ByteBuf};

use crate::rmps::Serializer;
use rmpv::Value;
use rmpv::encode;
use rmpv::ext::to_value;

/// Tests that a `Value` is properly encoded using two different mechanisms: direct serialization
/// using `rmp::encode::write_value` and using `serde`.
fn test_encode(v: Value, expected: &[u8]) {
    let mut buf0 = Vec::new();
    encode::write_value(&mut buf0, &v).unwrap();
    assert_eq!(expected, &buf0[..]);

    let mut buf1 = Vec::new();
    v.serialize(&mut Serializer::new(&mut buf1)).unwrap();
    assert_eq!(expected, &buf1[..]);
}

#[test]
fn pass_nil() {
    test_encode(Value::Nil, &[0xc0]);
}

#[test]
fn pass_bool() {
    test_encode(Value::Boolean(true), &[0xc3]);
    test_encode(Value::Boolean(false), &[0xc2]);
}

#[test]
fn pass_uint() {
    test_encode(Value::from(u8::min_value()), &[0x00]);
    test_encode(Value::from(u8::max_value()), &[0xcc, 0xff]);
    test_encode(Value::from(u16::max_value()), &[0xcd, 0xff, 0xff]);
    test_encode(Value::from(u32::max_value()), &[0xce, 0xff, 0xff, 0xff, 0xff]);
    test_encode(Value::from(u64::max_value()), &[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn pass_sint() {
    test_encode(Value::from(i8::min_value()), &[0xd0, 0x80]);
    test_encode(Value::from(i8::max_value()), &[0x7f]);
    test_encode(Value::from(i16::min_value()), &[0xd1, 0x80, 0x00]);
    test_encode(Value::from(i16::max_value()), &[0xcd, 0x7f, 0xff]);
    test_encode(Value::from(i32::min_value()), &[0xd2, 0x80, 0x00, 0x00, 0x00]);
    test_encode(Value::from(i32::max_value()), &[0xce, 0x7f, 0xff, 0xff, 0xff]);
    test_encode(Value::from(i64::min_value()), &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    test_encode(Value::from(i64::max_value()), &[0xcf, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
}

#[test]
fn pass_f32() {
    test_encode(Value::from(3.4028234e38f32), &[0xca, 0x7f, 0x7f, 0xff, 0xff]);
}

#[test]
fn pass_f64() {
    test_encode(Value::from(0.00), &[0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    test_encode(Value::from(42.0), &[0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn pass_str() {
    test_encode(Value::from("le message"),
        &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65]);
    test_encode(Value::from("le message".to_string()),
        &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65]);
    test_encode(Value::from(Cow::from("le message")),
        &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65]);
}

#[test]
fn pass_bin() {
    test_encode(Value::from(&[0xcc, 0x80][..]), &[0xc4, 0x02, 0xcc, 0x80]);
    test_encode(Value::from(vec![0xcc, 0x80]), &[0xc4, 0x02, 0xcc, 0x80]);
    test_encode(Value::from(Cow::from(&[0xcc, 0x80][..])), &[0xc4, 0x02, 0xcc, 0x80]);
}

#[test]
fn pass_array() {
    test_encode(Value::Array(vec![Value::from("le"), Value::from("shit")]),
        &[0x92, 0xa2, 0x6c, 0x65, 0xa4, 0x73, 0x68, 0x69, 0x74]);
}

#[test]
fn pass_value_map() {
    let val = Value::Map(vec![
        (Value::from(0), Value::from("le")),
        (Value::from(1), Value::from("shit")),
    ]);

    test_encode(val, &[0x82, 0x00, 0xa2, 0x6c, 0x65, 0x01, 0xa4, 0x73, 0x68, 0x69, 0x74]);
}

#[test]
fn pass_uint_to_value() {
    assert_eq!(Value::from(i8::min_value()), to_value(i8::min_value()).unwrap());
    assert_eq!(Value::from(i8::max_value()), to_value(i8::max_value()).unwrap());
    assert_eq!(Value::from(i16::min_value()), to_value(i16::min_value()).unwrap());
    assert_eq!(Value::from(i16::max_value()), to_value(i16::max_value()).unwrap());
    assert_eq!(Value::from(i32::min_value()), to_value(i32::min_value()).unwrap());
    assert_eq!(Value::from(i32::max_value()), to_value(i32::max_value()).unwrap());
    assert_eq!(Value::from(i64::min_value()), to_value(i64::min_value()).unwrap());
    assert_eq!(Value::from(i64::max_value()), to_value(i64::max_value()).unwrap());
}

#[test]
fn pass_sint_to_value() {
    assert_eq!(Value::from(0), to_value(0).unwrap());
    assert_eq!(Value::from(u8::max_value()), to_value(u8::max_value()).unwrap());
    assert_eq!(Value::from(u16::max_value()), to_value(u16::max_value()).unwrap());
    assert_eq!(Value::from(u32::max_value()), to_value(u32::max_value()).unwrap());
    assert_eq!(Value::from(u64::max_value()), to_value(u64::max_value()).unwrap());
}

#[test]
fn pass_f32_to_value() {
    assert_eq!(Value::from(0.0f32), to_value(0.0f32).unwrap());
    assert_eq!(Value::from(std::f32::consts::PI), to_value(std::f32::consts::PI).unwrap());
}

#[test]
fn pass_f64_to_value() {
    assert_eq!(Value::from(0.0), to_value(0.0).unwrap());
    assert_eq!(Value::from(std::f64::consts::PI), to_value(std::f64::consts::PI).unwrap());
}

#[test]
fn pass_char_to_value() {
    assert_eq!(Value::from("c"), to_value('c').unwrap());
}

#[test]
fn pass_str_to_value() {
    assert_eq!(Value::from("le message"), to_value("le message").unwrap());
    assert_eq!(Value::from("le message"), to_value("le message".to_string()).unwrap());
    assert_eq!(Value::from("le message"), to_value(Cow::from("le message")).unwrap());
}

#[test]
fn pass_bin_to_value() {
    assert_eq!(Value::from(vec![0, 1, 2]), to_value(Bytes::new(&[0, 1, 2])).unwrap());
    assert_eq!(Value::from(vec![0, 1, 2]), to_value(ByteBuf::from(&[0, 1, 2][..])).unwrap());
}

#[test]
fn pass_vec_to_value() {
    assert_eq!(Value::from(vec![Value::from("John"), Value::from("Smith")]),
        to_value(vec!["John", "Smith"]).unwrap());
}

#[test]
fn pass_map_to_value() {
    let mut map = BTreeMap::new();
    map.insert("name", "John");
    map.insert("surname", "Smith");

    assert_eq!(Value::from(vec![
        (Value::from("name"), Value::from("John")),
        (Value::from("surname"), Value::from("Smith"))
    ]), to_value(map).unwrap());
}

#[test]
fn pass_option_to_value() {
    assert_eq!(Value::Nil, to_value(None::<i32>).unwrap());
    assert_eq!(Value::Nil, to_value(Some(None::<i32>)).unwrap());
    assert_eq!(Value::from(42), to_value(Some(42)).unwrap());
    assert_eq!(Value::from(42), to_value(Some(Some(42))).unwrap());
}

#[test]
fn pass_seq_to_value() {
    assert_eq!(Value::Array(vec![Value::from(0), Value::from(42)]),
        to_value([0, 42]).unwrap());
}

#[test]
fn pass_tuple_to_value() {
    assert_eq!(Value::Array(vec![Value::from("John"), Value::from(42)]),
        to_value(("John", 42)).unwrap());
}

#[test]
fn pass_unit_struct_to_value() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Unit;

    assert_eq!(Value::Array(vec![]), to_value(Unit).unwrap());
}

#[test]
fn pass_newtype_struct_to_value() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Newtype(String);

    assert_eq!(Value::from("John"), to_value(Newtype("John".into())).unwrap());
}

#[test]
fn pass_tuple_struct_to_value() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Newtype(String, u8);

    assert_eq!(Value::Array(vec![Value::from("John"), Value::from(42)]),
        to_value(Newtype("John".into(), 42)).unwrap());
}

#[test]
fn pass_struct_to_value() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Struct {
        name: String,
        age: u8
    }

    assert_eq!(Value::Array(vec![Value::from("John"), Value::from(42)]),
        to_value(Struct { name: "John".into(), age: 42 }).unwrap());
}

#[test]
fn pass_enum_to_value() {
    #[derive(Debug, PartialEq, Serialize)]
    enum Enum {
        Unit,
        Newtype(String),
        Tuple(String, u32),
        Struct { name: String, age: u32 },
    }

    assert_eq!(Value::Array(vec![Value::from(0), Value::Array(vec![])]),
        to_value(Enum::Unit).unwrap());
    assert_eq!(Value::Array(vec![Value::from(1), Value::Array(vec![Value::from("John")])]),
        to_value(Enum::Newtype("John".into())).unwrap());
    assert_eq!(Value::Array(vec![Value::from(2), Value::Array(vec![Value::from("John"), Value::from(42)])]),
        to_value(Enum::Tuple("John".into(), 42)).unwrap());
    assert_eq!(Value::Array(vec![Value::from(3), Value::Array(vec![Value::from("John"), Value::from(42)])]),
        to_value(Enum::Struct { name: "John".into(), age: 42 }).unwrap());
}

#[test]
fn pass_ext_struct_to_value() {
    use serde_bytes::ByteBuf;

    #[derive(Debug, PartialEq, Serialize)]
    #[serde(rename = "_ExtStruct")]
    struct ExtStruct((i8, ByteBuf));

    assert_eq!(Value::Ext(5, vec![10]),
        to_value(ExtStruct((5, ByteBuf::from(vec![10])))).unwrap());
}
