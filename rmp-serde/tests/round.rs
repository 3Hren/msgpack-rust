#[macro_use]
extern crate serde_derive;

extern crate rmp_serde as rmps;

use std::borrow::Cow;
use std::io::Cursor;

use serde::{Deserialize, Serialize};
use crate::rmps::{Deserializer, Serializer};

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
fn round_struct_like_enum() {
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Enum {
        A { data: u32 },
    }

    let expected = Enum::A { data: 42 };
    let mut buf = Vec::new();
    expected.serialize(&mut Serializer::new(&mut buf)).unwrap();

    let mut de = Deserializer::new(&buf[..]);

    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}

#[test]
fn round_struct_like_enum_with_struct_map() {
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Enum {
        A { data: u32 },
    }

    let expected = Enum::A { data: 42 };
    let mut buf = Vec::new();
    expected
        .serialize(&mut Serializer::new(&mut buf).with_struct_map())
        .unwrap();

    let mut de = Deserializer::new(&buf[..]);

    assert_eq!(expected, Deserialize::deserialize(&mut de).unwrap());
}

#[test]
fn round_struct_like_enum_with_struct_tuple() {
    use serde::Serialize;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum Enum {
        A { data: u32 },
    }

    let expected = Enum::A { data: 42 };
    let mut buf = Vec::new();
    expected
        .serialize(&mut Serializer::new(&mut buf).with_struct_tuple())
        .unwrap();

    let mut de = Deserializer::new(&buf[..]);

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
    use crate::rmps::to_vec_named;
    use crate::rmps::decode::from_slice;

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

#[test]
fn round_struct_as_map_in_vec() {
    // See: issue #205
    use crate::rmps::decode::from_slice;
    use crate::rmps::to_vec_named;

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

    let data = vec![dog1];

    let serialized: Vec<u8> = to_vec_named(&data).unwrap();
    let deserialized: Vec<Dog2> = from_slice(&serialized).unwrap();

    let dog2 = &deserialized[0];

    assert_eq!(dog2.name, "Frankie");
    assert_eq!(dog2.age, 42);
}

#[test]
fn round_trip_unit_struct() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Message1 {
        data: u8,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Message2;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    enum Messages {
        Message1(Message1),
        Message2(Message2),
    }

    let msg2 = Messages::Message2(Message2);

    // struct-as-tuple
    {
        let serialized: Vec<u8> = rmps::to_vec(&msg2).unwrap();
        let deserialized: Messages = rmps::from_slice(&serialized).unwrap();
        assert_eq!(deserialized, msg2);
    }

    // struct-as-map
    {
        let serialized: Vec<u8> = rmps::to_vec_named(&msg2).unwrap();
        let deserialized: Messages = rmps::from_slice(&serialized).unwrap();
        assert_eq!(deserialized, msg2);
    }
}

#[test]
#[ignore]
fn round_trip_unit_struct_untagged_enum() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct UnitStruct;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct MessageA {
        some_int: i32,
        unit: UnitStruct,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    #[serde(untagged)]
    enum Messages {
        MessageA(MessageA),
    }

    let msga = Messages::MessageA(MessageA {
        some_int: 32,
        unit: UnitStruct,
    });

    // struct-as-tuple
    {
        let serialized: Vec<u8> = rmps::to_vec(&msga).unwrap();
        let deserialized: Messages = rmps::from_slice(&serialized).unwrap();
        assert_eq!(deserialized, msga);
    }

    // struct-as-map
    {
        let serialized: Vec<u8> = rmps::to_vec_named(&msga).unwrap();
        let deserialized: Messages = rmps::from_slice(&serialized).unwrap();
        assert_eq!(deserialized, msga);
    }
}

// Checks whether deserialization and serialization can both work with enum variants as strings
#[test]
fn round_variant_string() {
    use crate::rmps::decode::from_slice;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    enum Animal1 {
        Dog { breed: String },
        Cat,
        Emu,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    enum Animal2 {
        Emu,
        Dog { breed: String },
        Cat,
    }

    // use helper macro so that we can test many combinations at once. Needs to be a macro to deal
    // with the serializer owning a reference to the Vec.
    macro_rules! do_test {
        ($ser:expr) => {
            {
                let animal1 = Animal1::Dog { breed: "Pitbull".to_owned() };
                let expected = Animal2::Dog { breed: "Pitbull".to_owned() };
                let mut buf = Vec::new();
                animal1.serialize(&mut $ser(&mut buf)).unwrap();

                let deserialized: Animal2 = from_slice(&buf).unwrap();
                assert_eq!(deserialized, expected);
            }
        }
    }

    do_test!(|b| Serializer::new(b).with_string_variants());
    do_test!(|b| Serializer::new(b).with_struct_map().with_string_variants());
    do_test!(|b| Serializer::new(b).with_struct_tuple().with_string_variants());
    do_test!(|b| Serializer::new(b).with_string_variants().with_struct_map());
    do_test!(|b| Serializer::new(b).with_string_variants().with_struct_tuple());
    do_test!(|b| {
        Serializer::new(b)
            .with_string_variants()
            .with_struct_tuple()
            .with_struct_map()
            .with_struct_tuple()
            .with_struct_map()
    });
    do_test!(|b| Serializer::new(b).with_integer_variants().with_string_variants());
}
