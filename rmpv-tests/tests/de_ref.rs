#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use std::collections::BTreeMap;

use serde_bytes::ByteBuf;

use rmpv::ValueRef;
use rmpv::decode;
use rmpv::ext::deserialize_from;

/// Tests that a `ValueRef` is properly decoded from bytes using two different mechanisms: direct
/// deserialization using `rmp::decode::read_value_ref` and using `serde`.
fn test_decode(buf: &[u8], v: ValueRef<'_>) {
    let val0: ValueRef<'_> = decode::read_value_ref(&mut &buf[..]).unwrap();
    assert_eq!(v, val0);

    let val1: ValueRef<'_> = rmps::from_slice(buf).unwrap();
    assert_eq!(v, val1);
}

#[test]
fn pass_null() {
    test_decode(&[0xc0], ValueRef::Nil);
}

#[test]
fn pass_bool() {
    test_decode(&[0xc3], ValueRef::Boolean(true));
    test_decode(&[0xc2], ValueRef::Boolean(false));
}

#[test]
fn pass_uint() {
    test_decode(&[0x00], ValueRef::from(u8::min_value()));
    test_decode(&[0xcc, 0xff], ValueRef::from(u8::max_value()));
    test_decode(&[0xcd, 0xff, 0xff], ValueRef::from(u16::max_value()));
    test_decode(&[0xce, 0xff, 0xff, 0xff, 0xff], ValueRef::from(u32::max_value()));
    test_decode(&[0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], ValueRef::from(u64::max_value()));
}

#[test]
fn pass_sint() {
    test_decode(&[0xd0, 0x80], ValueRef::from(i8::min_value()));
    test_decode(&[0x7f], ValueRef::from(i8::max_value()));
    test_decode(&[0xd1, 0x80, 0x00], ValueRef::from(i16::min_value()));
    test_decode(&[0xcd, 0x7f, 0xff], ValueRef::from(i16::max_value()));
    test_decode(&[0xd2, 0x80, 0x00, 0x00, 0x00], ValueRef::from(i32::min_value()));
    test_decode(&[0xce, 0x7f, 0xff, 0xff, 0xff], ValueRef::from(i32::max_value()));
    test_decode(&[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], ValueRef::from(i64::min_value()));
    test_decode(&[0xcf, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], ValueRef::from(i64::max_value()));
}

#[test]
fn pass_f32() {
    test_decode(&[0xca, 0x7f, 0x7f, 0xff, 0xff], ValueRef::from(3.4028234e38f32));
}

#[test]
fn pass_f64() {
    test_decode(&[0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], ValueRef::from(0.00));
    test_decode(&[0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], ValueRef::from(42.0));
}

#[test]
fn pass_str() {
    test_decode(&[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65],
                ValueRef::from("le message"));
}

#[test]
fn pass_bin() {
    test_decode(&[0xc4, 0x02, 0xcc, 0x80], ValueRef::from(&[0xcc, 0x80][..]));
}

#[test]
fn pass_array() {
    test_decode(&[0x92, 0xa2, 0x6c, 0x65, 0xa4, 0x73, 0x68, 0x69, 0x74],
                ValueRef::Array(vec![ValueRef::from("le"), ValueRef::from("shit")]));
}

#[test]
fn pass_value_map() {
    let val = ValueRef::Map(vec![
        (ValueRef::from(0), ValueRef::from("le")),
        (ValueRef::from(1), ValueRef::from("shit")),
    ]);

    test_decode(&[0x82, 0x00, 0xa2, 0x6c, 0x65, 0x01, 0xa4, 0x73, 0x68, 0x69, 0x74], val);
}

#[test]
fn pass_uint_from_value() {
    assert_eq!(i8::min_value(), deserialize_from(ValueRef::from(i8::min_value())).unwrap());
    assert_eq!(i8::max_value(), deserialize_from(ValueRef::from(i8::max_value())).unwrap());
    assert_eq!(i16::min_value(), deserialize_from(ValueRef::from(i16::min_value())).unwrap());
    assert_eq!(i16::max_value(), deserialize_from(ValueRef::from(i16::max_value())).unwrap());
    assert_eq!(i32::min_value(), deserialize_from(ValueRef::from(i32::min_value())).unwrap());
    assert_eq!(i32::max_value(), deserialize_from(ValueRef::from(i32::max_value())).unwrap());
    assert_eq!(i64::min_value(), deserialize_from(ValueRef::from(i64::min_value())).unwrap());
    assert_eq!(i64::max_value(), deserialize_from(ValueRef::from(i64::max_value())).unwrap());
}

#[test]
fn pass_sint_from_value() {
    assert_eq!(0, deserialize_from(ValueRef::from(0)).unwrap());
    assert_eq!(u8::max_value(), deserialize_from(ValueRef::from(u8::max_value())).unwrap());
    assert_eq!(u16::max_value(), deserialize_from(ValueRef::from(u16::max_value())).unwrap());
    assert_eq!(u32::max_value(), deserialize_from(ValueRef::from(u32::max_value())).unwrap());
    assert_eq!(u64::max_value(), deserialize_from(ValueRef::from(u64::max_value())).unwrap());
}

#[test]
fn pass_f32_from_value() {
    assert_eq!(0.0f32, deserialize_from(ValueRef::from(0.0f32)).unwrap());
    assert_eq!(std::f32::consts::PI, deserialize_from(ValueRef::from(std::f32::consts::PI)).unwrap());
}

#[test]
fn pass_f64_from_value() {
    assert_eq!(0.0, deserialize_from(ValueRef::from(0.0)).unwrap());
    assert_eq!(std::f64::consts::PI, deserialize_from(ValueRef::from(std::f64::consts::PI)).unwrap());
}

#[test]
fn pass_char_from_value() {
    assert_eq!('c', deserialize_from(ValueRef::from("c")).unwrap());
}

#[test]
fn pass_string_from_value() {
    let v: String = deserialize_from(ValueRef::from("le message")).unwrap();
    assert_eq!("le message".to_string(), v);

    let v: &str = deserialize_from(ValueRef::from("le message")).unwrap();
    assert_eq!("le message", v);
}

#[test]
fn pass_bin_from_value() {
    let buf = &[0, 1, 2];
    let v: &[u8] = deserialize_from(ValueRef::from(&buf[..])).unwrap();
    assert_eq!(&[0, 1, 2][..], v);

    assert_eq!(
        ByteBuf::from(&[0, 1, 2][..]),
        deserialize_from::<ByteBuf, _>(ValueRef::from(&[0, 1, 2][..])).unwrap()
    );
}

#[test]
fn pass_vec_from_value() {
    let v: Vec<&str> = deserialize_from(ValueRef::from(vec![ValueRef::from("John"), ValueRef::from("Smith")])).unwrap();
    assert_eq!(vec!["John", "Smith"], v);
}


#[test]
fn pass_map_from_value() {
    let mut map = BTreeMap::new();
    map.insert("name", "John");
    map.insert("surname", "Smith");

    let val = ValueRef::from(vec![
        (ValueRef::from("name"), ValueRef::from("John")),
        (ValueRef::from("surname"), ValueRef::from("Smith"))
    ]);

    let v: BTreeMap<&str, &str> = deserialize_from(val).unwrap();

    assert_eq!(map, v);
}

#[test]
fn pass_option_from_value() {
    assert_eq!(None::<i32>, deserialize_from(ValueRef::Nil).unwrap());
    // TODO: assert_eq!(Some(None::<i32>), from_value(ValueRef::Nil).unwrap());
    assert_eq!(Some(42), deserialize_from(ValueRef::from(42)).unwrap());
    assert_eq!(Some(Some(42)), deserialize_from(ValueRef::from(42)).unwrap());
}

#[test]
fn pass_seq_from_value() {
    let v: Vec<u64> = deserialize_from(ValueRef::Array(vec![ValueRef::from(0), ValueRef::from(42)])).unwrap();
    assert_eq!(vec![0, 42], v);
}

#[test]
fn pass_tuple_from_value() {
    let v: (&str, u8) = deserialize_from(ValueRef::Array(vec![ValueRef::from("John"), ValueRef::from(42)])).unwrap();
    assert_eq!(("John", 42), v);
}

#[test]
fn pass_unit_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Unit;

    assert_eq!(Unit, deserialize_from(ValueRef::Array(vec![])).unwrap());
}

#[test]
fn pass_newtype_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Newtype<'a>(&'a str);

    assert_eq!(Newtype("John"), deserialize_from(ValueRef::from("John")).unwrap());
}

#[test]
fn pass_tuple_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Newtype<'a>(&'a str, u8);

    assert_eq!(Newtype("John", 42),
        deserialize_from(ValueRef::Array(vec![ValueRef::from("John"), ValueRef::from(42)])).unwrap());
}

#[test]
fn pass_struct_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Struct<'a> {
        name: &'a str,
        age: u8
    }

    assert_eq!(Struct { name: "John", age: 42 },
        deserialize_from(ValueRef::Array(vec![ValueRef::from("John"), ValueRef::from(42)])).unwrap());
}

#[test]
fn pass_enum_from_value() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum<'a> {
        Unit,
        Newtype(&'a str),
        Tuple(&'a str , u32),
        Struct { name: &'a str, age: u32 },
    }

    assert_eq!(Enum::Unit,
        deserialize_from(ValueRef::Array(vec![ValueRef::from(0), ValueRef::Array(vec![])])).unwrap());
    assert_eq!(Enum::Newtype("John"),
        deserialize_from(ValueRef::Array(vec![ValueRef::from(1), ValueRef::Array(vec![ValueRef::from("John")])])).unwrap());
    assert_eq!(Enum::Tuple("John", 42),
        deserialize_from(ValueRef::Array(vec![ValueRef::from(2), ValueRef::Array(vec![ValueRef::from("John"), ValueRef::from(42)])])).unwrap());
    assert_eq!(Enum::Struct { name: "John", age: 42 },
        deserialize_from(ValueRef::Array(vec![ValueRef::from(3), ValueRef::Array(vec![ValueRef::from("John"), ValueRef::from(42)])])).unwrap());
}

#[test]
fn pass_from_slice() {
    let buf = [0x93, 0xa4, 0x4a, 0x6f, 0x68, 0x6e, 0xa5, 0x53, 0x6d, 0x69, 0x74, 0x68, 0x2a];

    assert_eq!(ValueRef::Array(vec![ValueRef::from("John"), ValueRef::from("Smith"), ValueRef::from(42)]),
        rmps::from_slice(&buf[..]).unwrap());
}

#[test]
fn pass_from_ext() {
    #[derive(Debug, PartialEq)]
    struct ExtRefStruct<'a>(i8, &'a [u8]);

    struct ExtRefStructVisitor;

    impl<'de> serde::de::Deserialize<'de> for ExtRefStruct<'de> {
        fn deserialize<D>(deserializer: D) -> Result<ExtRefStruct<'de>, D::Error>
            where D: serde::Deserializer<'de>,
        {
            let visitor = ExtRefStructVisitor;
            deserializer.deserialize_any(visitor)
        }
    }

    impl<'de> serde::de::Visitor<'de> for ExtRefStructVisitor {
        type Value = ExtRefStruct<'de>;

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
            let data: &[u8] = seq.next_element()?
                .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

            Ok(ExtRefStruct(tag, data))
        }
    }

    assert_eq!(ExtRefStruct(42, &[255]),
        deserialize_from(ValueRef::Ext(42, &[255])).unwrap());
}
