#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

extern crate serde;
extern crate rmp;
extern crate rmp_serde;

use std::io::Cursor;
use std::result;

use serde::Deserialize;

use rmp_serde::Deserializer;
use rmp_serde::decode::Error;

type Result<T> = result::Result<T, Error>;

#[cfg(feature = "serde_macros")]
#[test]
fn pass_tuple_struct() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Decoded(u32, u32);

    let mut deserializer = Deserializer::new(cur);
    let actual: Decoded = Deserialize::deserialize(&mut deserializer).unwrap();

    assert_eq!(Decoded(42, 100500), actual);
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_struct() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    struct Decoded { id: u32, value: u32 };

    let mut deserializer = Deserializer::new(cur);
    let actual: Decoded = Deserialize::deserialize(&mut deserializer).unwrap();

    assert_eq!(Decoded { id: 42, value: 100500 }, actual);
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_struct_map() {
    #[derive(Debug, PartialEq, Deserialize)]
    struct Custom {
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

    // it appears no special behavior is needed for deserializing structs encoded as maps
    let mut deserializer = Deserializer::new(cur);
    let actual: Custom = Deserialize::deserialize(&mut deserializer).unwrap();
    let voila = "voila".to_string(); // so the next line looks more funny
    let expected = Custom { et: voila, le: 0, shit: 1 };

    assert_eq!(expected, actual);
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_enum() {
    // We expect enums to be endoded as [id, [...]]

    let buf = [0x92, 0x01, 0x90];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Custom {
        First,
        Second,
    }

    let mut de = Deserializer::new(cur);
    let actual: Custom = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Custom::Second, actual);
    assert_eq!(3, de.get_ref().position());
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_enum_variant_with_arg() {
    // The encoded bytearray is: [1, [42]].
    let buf = [0x92, 0x01, 0x91, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Custom {
        First,
        Second(u32),
    }

    let mut de = Deserializer::new(cur);
    let actual: Custom = Deserialize::deserialize(&mut de).unwrap();

    assert_eq!(Custom::Second(42), actual);
    assert_eq!(4, de.get_ref().position())
}

#[cfg(feature = "serde_macros")]
#[test]
fn fail_enum_sequence_mismatch() {
    let buf = [0x93, 0x1, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Custom {
        First,
        Second,
    }

    let mut deserializer = Deserializer::new(cur);
    let actual: Result<Custom> = Deserialize::deserialize(&mut deserializer);

    match actual.err() {
        Some(Error::Uncategorized(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[cfg(feature = "serde_macros")]
#[test]
fn fail_enum_overflow() {
    let buf = [0x92, 0x01, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Custom {
        First,
    }

    let mut deserializer = Deserializer::new(cur);
    let actual: Result<Custom> = Deserialize::deserialize(&mut deserializer);

    match actual.err() {
        Some(Error::Uncategorized(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_struct_enum_with_arg() {
    let buf = [0x92, 0x01, 0x91, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, Deserialize)]
    enum Custom {
        First,
        Second { id: u32 },
    }

    let mut deserializer = Deserializer::new(cur);
    let actual: Custom = Deserialize::deserialize(&mut deserializer).unwrap();

    assert_eq!(Custom::Second { id: 42 }, actual);
}
