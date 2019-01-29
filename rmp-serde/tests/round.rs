extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate rmp;
extern crate rmp_serde as rmps;

use std::borrow::Cow;
use std::io::Cursor;

use serde::{Deserialize, Serialize};
use rmps::{Deserializer, Serializer};

#[test]
fn round_trip_option() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        v: Option<Vec<u8>>,
    }

    let expected = Foo { v: None };

    let mut buf = Vec::new();
    expected.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let mut de = Deserializer::new(Cursor::new(&buf[..]));

    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}

#[test]
fn round_trip_optional_enum() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub enum SimpleEnum {
        Variant,
    }
    let expected = Some(SimpleEnum::Variant);

    let mut buf = Vec::new();
    expected.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let mut de = Deserializer::new(Cursor::new(&buf[..]));
    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}

#[test]
fn round_trip_cow() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Foo<'a> {
        v: Cow<'a, [u8]>,
    }

    let expected = Foo { v : Cow::Borrowed(&[]) };

    let mut buf = Vec::new();
    expected.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let mut de = Deserializer::new(Cursor::new(&buf[..]));

    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}

#[test]
fn round_trip_option_cow() {
    use std::borrow::Cow;
    use std::io::Cursor;
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Foo<'a> {
        v: Option<Cow<'a, [u8]>>,
    }

    let expected = Foo { v : None };

    let mut buf = Vec::new();
    expected.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let mut de = Deserializer::new(Cursor::new(&buf[..]));

    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}

#[test]
fn round_enum_with_newtype_struct() {
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Newtype(String);

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Enum {
        A(Newtype),
    }

    let expected = Enum::A(Newtype("le message".into()));
    let mut buf = Vec::new();
    expected.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let mut de = Deserializer::new(&buf[..]);

    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}

#[test]
fn round_trip_untagged_enum_with_enum_associated_data() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    #[serde(untagged)]
    enum Foo {
        A(Bar),
    }

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Bar {
        B,
        C(String),
        D(u64, u64, u64),
        E{f1: String},
    }

    let data1_1 = Foo::A(Bar::B);
    let bytes_1 = rmps::to_vec(&data1_1).unwrap();
    let data1_2 = rmps::from_slice(&bytes_1).unwrap();
    assert_eq!(data1_1, data1_2);

    let data2_1 = Foo::A(Bar::C("Hello".into()));
    let bytes_2 = rmps::to_vec(&data2_1).unwrap();
    let data2_2 = rmps::from_slice(&bytes_2).unwrap();
    assert_eq!(data2_1, data2_2);

    let data3_1 = Foo::A(Bar::D(1,2,3));
    let bytes_3 = rmps::to_vec(&data3_1).unwrap();
    let data3_2 = rmps::from_slice(&bytes_3).unwrap();
    assert_eq!(data3_1, data3_2);

    let data4_1 = Foo::A(Bar::E{f1: "Hello".into()});
    let bytes_4 = rmps::to_vec(&data4_1).unwrap();
    let data4_2 = rmps::from_slice(&bytes_4).unwrap();
    assert_eq!(data4_1, data4_2);
}

// Checks whether deserialization and serialization can both work with structs as maps
#[test]
fn round_struct_as_map() {
    use rmps::to_vec_named;
    use rmps::decode::from_slice;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Dog1 {
        name: String,
        age: u16,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Dog2 {
        age: u16,
        name: String,
    }

    let dog1 = Dog1 {
        name: "Frankie".into(),
        age: 42,
    };

    let serialized: Vec<u8> = to_vec_named(&dog1).unwrap();
    let deserialized: Dog2 = from_slice(&serialized).unwrap();

    let check = Dog1 {
        age: deserialized.age,
        name: deserialized.name,
    };

    assert_eq!(dog1, check);
}
