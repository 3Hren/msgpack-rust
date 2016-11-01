use std::borrow::Cow;
use std::io::Cursor;
use serde::Serialize;

#[test]
fn round_trip_option() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        v: Option<Vec<u8>>,
    }

    let expected = Foo { v: None };

    let mut data = vec![];
    expected.serialize(&mut rmp_serde::encode::Serializer::new(&mut data)).unwrap();

    let mut de = rmp_serde::decode::Deserializer::new(Cursor::new(&data[..]));
    let actual: Foo = serde::Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn round_trip_cow() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Foo<'a> {
        v: Cow<'a, [u8]>,
    }

    let expected = Foo { v : Cow::Borrowed(&[]) };

    let mut data = vec![];
    expected.serialize(&mut rmp_serde::encode::Serializer::new(&mut data)).unwrap();

    let mut de = rmp_serde::decode::Deserializer::new(Cursor::new(&data[..]));
    let actual: Foo = serde::Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(expected, actual);
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

    let mut data = vec![];
    expected.serialize(&mut rmp_serde::encode::Serializer::new(&mut data)).unwrap();

    let mut de = rmp_serde::decode::Deserializer::new(Cursor::new(&data[..]));
    let actual: Foo = serde::Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(expected, actual);
}
