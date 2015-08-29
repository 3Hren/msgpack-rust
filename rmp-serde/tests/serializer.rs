#![cfg_attr(feature = "serde_macros", feature(custom_derive, plugin))]
#![cfg_attr(feature = "serde_macros", plugin(serde_macros))]

extern crate serde;
extern crate rmp_serde;

use std::io::Cursor;

use serde::Serialize;

use rmp_serde::Serializer;
use rmp_serde::encode::Error;

#[test]
fn pass_null() {
    let mut buf = [0x00];

    let val = ();
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xc0], buf);
}

#[test]
fn fail_null() {
    let mut buf = [];

    let val = ();

    match val.serialize(&mut Serializer::new(&mut &mut buf[..])) {
        Err(Error::InvalidFixedValueWrite(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_bool() {
    let mut buf = [0x00, 0x00];

    {
        let mut cur = Cursor::new(&mut buf[..]);

        let mut encoder = Serializer::new(&mut cur);

        let val = true;
        val.serialize(&mut encoder).ok().unwrap();
        let val = false;
        val.serialize(&mut encoder).ok().unwrap();
    }

    assert_eq!([0xc3, 0xc2], buf);
}

#[test]
fn pass_usize() {
    let mut buf = [0x00, 0x00];

    let val = 255usize;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcc, 0xff], buf);
}

#[test]
fn pass_u8() {
    let mut buf = [0x00, 0x00];

    let val = 255u8;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcc, 0xff], buf);
}

#[test]
fn pass_u16() {
    let mut buf = [0x00, 0x00, 0x00];

    let val = 65535u16;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcd, 0xff, 0xff], buf);
}

#[test]
fn pass_u32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 4294967295u32;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xce, 0xff, 0xff, 0xff, 0xff], buf);
}

#[test]
fn pass_u64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 18446744073709551615u64;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], buf);
}

#[test]
fn pass_isize() {
    let mut buf = [0x00, 0x00];

    let val = -128isize;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd0, 0x80], buf);
}

#[test]
fn pass_i8() {
    let mut buf = [0x00, 0x00];

    let val = -128i8;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd0, 0x80], buf);
}

#[test]
fn pass_i16() {
    let mut buf = [0x00, 0x00, 0x00];

    let val = -32768i16;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd1, 0x80, 0x00], buf);
}

#[test]
fn pass_i32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    let val = -2147483648i32;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd2, 0x80, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_i64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = -9223372036854775808i64;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_f32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 3.4028234e38_f32;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xca, 0x7f, 0x7f, 0xff, 0xff], buf);
}

#[test]
fn pass_f64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 42f64;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_char() {
    let mut buf = [0x00, 0x00];

    let val = '!';
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xa1, 0x21], buf);
}


#[test]
fn pass_string() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = "le message";
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65], buf);
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_struct() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[derive(Serialize)]
    struct Decoded { id: u32, value: u32 }

    let val = Decoded { id: 42, value: 100500 };
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94], buf);
}

#[test]
fn pass_tuple() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = (42u32, 100500u32);
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94], buf);
}

#[test]
fn pass_option_some() {
    let mut buf = [0x00];

    let val = Some(100u32);
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0x64], buf);
}

#[test]
fn pass_option_none() {
    let mut buf = [0x00];

    let val: Option<u32> = None;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xc0], buf);
}

#[test]
fn pass_seq() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = vec!["le", "shit"];
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0x92, 0xa2, 0x6c, 0x65, 0xa4, 0x73, 0x68, 0x69, 0x74], buf);
}

#[test]
fn pass_map() {
    use std::collections::BTreeMap;

    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let mut val = BTreeMap::new();
    val.insert(0u8, "le");
    val.insert(1u8, "shit");
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    let out = [
        0x82, // 2 (size)
        0x00, // 0
        0xa2, 0x6c, 0x65, // "le"
        0x01, // 1
        0xa4, 0x73, 0x68, 0x69, 0x74, // "shit"
    ];
    assert_eq!(out, buf);
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_struct_map() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Custom<'a> {
        et: &'a str,
        le: u8,
        shit: u8,
    }

    let mut buf = [0x00; 20];

    let val = Custom { et: "voila", le: 0, shit: 1 };
    val.serialize(&mut Serializer::new_verbose(&mut &mut buf[..])).ok().unwrap();

    let out = [
        0x83, // 3 (size)
        0xa2, 0x65, 0x74, // "et"
        0xa5, 0x76, 0x6f, 0x69, 0x6c, 0x61, // "voila"
        0xa2, 0x6c, 0x65, // "le"
        0x00, // 0
        0xa4, 0x73, 0x68, 0x69, 0x74, // "shit"
        0x01, // 1
    ];
    assert_eq!(out, buf);
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_enum() {
    // We encode enum types as [id, [args...]].

    #[allow(unused)]
    #[derive(Debug, PartialEq, Serialize)]
    enum Custom {
        First,
        Second,
    }

    let mut buf = [0x00; 3];

    let val = Custom::Second;
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    let out = [0x92, 0x01, 0x90];
    assert_eq!(out, buf);
}

#[cfg(feature = "serde_macros")]
#[test]
fn pass_enum_variant_with_arg() {
    #[allow(unused)]
    #[derive(Debug, PartialEq, Serialize)]
    enum Custom {
        First,
        Second(u32),
    }

    let mut buf = [0x00; 4];

    let val = Custom::Second(42);
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    let out = [0x92, 0x01, 0x91, 0x2a];
    assert_eq!(out, buf);
}

#[test]
fn pass_encodong_struct_into_vec() {
    let val = (42u8, "the Answer");

    let mut buf: Vec<u8> = Vec::new();

    val.serialize(&mut Serializer::new(&mut buf)).unwrap();

    assert_eq!(vec![0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72], buf);
}

#[cfg(feature = "serde_macros")]
#[test]
fn encode_struct_with_string_using_vec() {
    #[derive(Debug, PartialEq, Serialize)]
    struct Custom {
        data: String,
    }

    let mut buf = Vec::new();

    let val = Custom { data: "le message".to_string() };
    val.serialize(&mut Serializer::new(&mut buf)).ok().unwrap();

    let out = vec![0x91, 0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    assert_eq!(out, buf);
}

#[test]
fn pass_bin() {
    use serde::bytes::Bytes;

    let mut buf = Vec::new();
    let vec = vec![0xcc, 0x80];
    let val = Bytes::from(&vec);

    val.serialize(&mut Serializer::new(&mut buf)).ok().unwrap();

    assert_eq!(vec![0xc4, 0x02, 0xcc, 0x80], buf);
}
