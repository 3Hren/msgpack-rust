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
fn round_enum_with_nested_struct() {
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Newtype(String);

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Enum {
        A(Newtype),
        B,
    }

    let expected = Enum::A(Newtype("le message".into()));
    let mut buf = Vec::new();
    expected.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let mut de = Deserializer::new(&buf[..]);

    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}
