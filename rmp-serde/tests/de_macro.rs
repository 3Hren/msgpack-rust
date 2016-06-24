#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

#![cfg(feature = "serde_macros")]

extern crate serde;
extern crate rmp;
extern crate rmp_serde;

use std::io::Cursor;
use std::result;

use serde::Deserialize;

use rmp_serde::Deserializer;
use rmp_serde::decode::Error;

type Result<T> = result::Result<T, Error>;

#[test]
fn pass_tuple_struct() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Decoded(u32, u32);

    let mut de = Deserializer::new(cur);
    let actual: Decoded = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Decoded(42, 100500), actual);
}

#[test]
fn pass_struct() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Decoded { id: u32, value: u32 };

    let mut de = Deserializer::new(cur);
    let actual: Decoded = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Decoded { id: 42, value: 100500 }, actual);
}

#[test]
fn pass_struct_map() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Struct {
        et: String,
        le: u8,
        shit: u8,
    }

    let buf = [
        0x83, // 3 (size)
        0xa2, 0x65, 0x74, // "et"
        0xa5, 0x76, 0x6f, 0x69, 0x6c, 0x61, // "voila"
        0xa2, 0x6c, 0x65, // "le"
        0x00, // 0
        0xa4, 0x73, 0x68, 0x69, 0x74, // "shit"
        0x01, // 1
    ];
    let cur = Cursor::new(&buf[..]);

    // It appears no special behavior is needed for deserializing structs encoded as maps.
    let mut de = Deserializer::new(cur);
    let actual: Struct = Deserialize::deserialize(&mut de).unwrap();
    let expected = Struct { et: "voila".into(), le: 0, shit: 1 };

    assert_eq!(expected, actual);
}

#[test]
fn pass_enum() {
    // We expect enums to be endoded as [id, [...]]

    let buf = [0x92, 0x01, 0x90];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        A,
        B,
    }

    let mut de = Deserializer::new(cur);
    let actual: Enum = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Enum::B, actual);
    assert_eq!(3, de.get_ref().position());
}

#[test]
fn pass_tuple_enum_with_arg() {
    // The encoded bytearray is: [1, [42]].
    let buf = [0x92, 0x01, 0x91, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        A,
        B(u32),
    }

    let mut de = Deserializer::new(cur);
    let actual: Enum = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Enum::B(42), actual);
    assert_eq!(4, de.get_ref().position())
}

#[test]
fn pass_tuple_enum_with_args() {
    // The encoded bytearray is: [1, [42, 58]].
    let buf = [0x92, 0x01, 0x92, 0x2a, 0x3a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        A,
        B(u32, u32),
    }

    let mut de = Deserializer::new(cur);
    let actual: Enum = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Enum::B(42, 58), actual);
    assert_eq!(5, de.get_ref().position())
}

#[test]
fn fail_enum_sequence_mismatch() {
    // The encoded bytearray is: [1, 2, 100500].
    let buf = [0x93, 0x1, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        A,
        B,
    }

    let mut de = Deserializer::new(cur);
    let actual: Result<Enum> = Deserialize::deserialize(&mut de);

    match actual.err().unwrap() {
        Error::LengthMismatch(3) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn fail_enum_overflow() {
    // The encoded bytearray is: [1, [42]].
    let buf = [0x92, 0x01, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    // TODO: Rename to Enum: A, B, C, ...
    enum Enum {
        A,
    }

    let mut de = Deserializer::new(cur);
    let actual: Result<Enum> = Deserialize::deserialize(&mut de);

    match actual.err().unwrap() {
        Error::Syntax(..) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_struct_enum_with_arg() {
    // The encoded bytearray is: [1, [42]].
    let buf = [0x92, 0x01, 0x91, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        A,
        B { id: u32 },
    }

    let mut de = Deserializer::new(cur);
    let actual: Enum = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Enum::B { id: 42 }, actual);
    assert_eq!(4, de.get_ref().position())
}

#[test]
fn pass_enum_with_nested_struct() {
    // The encoded bytearray is: [0, [['le message']]].
    let buf = [0x92, 0x0, 0x91, 0x91, 0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Nested(String);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        A(Nested),
        B,
    }

    let mut de = Deserializer::new(cur);
    let actual: Enum = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Enum::A(Nested("le message".into())), actual);
    assert_eq!(buf.len() as u64, de.get_ref().position())
}

#[test]
fn pass_enum_custom_policy() {
    use std::io::Read;
    use rmp_serde::decode::VariantVisitor;

    // We expect enums to be endoded as id, [...] (without wrapping tuple).

    let buf = [0x01, 0x90];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Enum {
        A,
        B,
    }

    struct CustomDeserializer<R: Read> {
        inner: Deserializer<R>,
    }

    impl<R: Read> serde::Deserializer for CustomDeserializer<R> {
        type Error = Error;

        fn deserialize<V>(&mut self, visitor: V) -> Result<V::Value>
            where V: serde::de::Visitor
        {
            self.inner.deserialize(visitor)
        }

        fn deserialize_enum<V>(&mut self, _enum: &str, _variants: &'static [&'static str], mut visitor: V)
            -> Result<V::Value>
            where V: serde::de::EnumVisitor
        {
            visitor.visit(VariantVisitor::new(&mut self.inner))
        }
    }

    let mut de = CustomDeserializer { inner: Deserializer::new(cur) };
    let actual: Enum = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Enum::B, actual);
    assert_eq!(2, de.inner.get_ref().position());
}

#[test]
fn pass_serialize_struct_variant() {
    #[derive(Debug, PartialEq, Deserialize)]
    enum Custom {
        First { data: u32 },
        Second { data: u32 },
    }
    let out_first = vec![0x92, 0x00, 0x91, 0x2a];
    let out_second = vec![0x92, 0x01, 0x91, 0x2a];

    for (expected, out) in vec![(Custom::First{ data: 42 }, out_first), (Custom::Second { data: 42 }, out_second)] {
        let mut de = Deserializer::new(Cursor::new(&out[..]));
        let val: Custom = Deserialize::deserialize(&mut de).unwrap();
        assert_eq!(expected, val);
    }
}
