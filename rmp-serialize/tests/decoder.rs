extern crate rmp as msgpack;
extern crate rmp_serialize;
extern crate rustc_serialize;

use std::io::Cursor;
use std::result;

use rustc_serialize::Decodable;

use msgpack::Marker;
use rmp_serialize::Decoder;
use rmp_serialize::decode::Error;

type Result<T> = result::Result<T, Error>;

#[test]
fn pass_null() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn fail_null_from_reserved() {
    let buf = [0xc1];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    let res: Result<()> = Decodable::decode(&mut decoder);
    match res.err() {
        Some(Error::TypeMismatch(Marker::Reserved)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_bool() {
    let buf = [0xc2, 0xc3];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(false, Decodable::decode(&mut decoder).ok().unwrap());
    assert_eq!(true,  Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn fail_bool_from_fixint() {
    let buf = [0x00];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    let res: Result<bool> = Decodable::decode(&mut decoder);
    match res.err() {
        Some(Error::TypeMismatch(Marker::FixPos(0))) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_u64() {
    let buf = [0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(18446744073709551615u64, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_u32() {
    let buf = [0xce, 0xff, 0xff, 0xff, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(4294967295u32, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn fail_u32_from_u64() {
    let buf = [0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    let res: Result<u32> = Decodable::decode(&mut decoder);
    match res.err() {
        Some(Error::TypeMismatch(Marker::U64)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_u16() {
    let buf = [0xcd, 0xff, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(65535u16, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_u8() {
    let buf = [0xcc, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(255u8, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_usize() {
    let buf = [0xcc, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(255usize, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_i64() {
    let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(9223372036854775807i64, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_i32() {
    let buf = [0xd2, 0x7f, 0xff, 0xff, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(2147483647i32, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_i16() {
    let buf = [0xd1, 0x7f, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(32767i16, Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_i8() {
    let buf = [0xd0, 0x7f];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(127i8, Decodable::decode(&mut decoder).unwrap());
}

#[test]
fn pass_isize() {
    let buf = [0xd0, 0x7f];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(127isize, Decodable::decode(&mut decoder).unwrap());
}

#[test]
fn pass_f32() {
    let buf = [0xca, 0x7f, 0x7f, 0xff, 0xff];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(3.4028234e38_f32, Decodable::decode(&mut decoder).unwrap());
}

#[test]
fn pass_f64() {
    let buf = [0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!(42f64, Decodable::decode(&mut decoder).unwrap());
}

#[test]
fn pass_string() {
    let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: String = Decodable::decode(&mut decoder).unwrap();

    assert_eq!("le message".to_string(), actual);
}

#[test]
fn pass_tuple() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: (u32, u32) = Decodable::decode(&mut decoder).unwrap();

    assert_eq!((42, 100500), actual);
}

#[test]
fn fail_tuple_len_mismatch() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: Result<(u32,)> = Decodable::decode(&mut decoder);

    match actual.err() {
        Some(Error::LengthMismatch(2)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_tuple_struct() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, RustcDecodable)]
    struct Decoded(u32, u32);

    let mut decoder = Decoder::new(cur);
    let actual: Decoded = Decodable::decode(&mut decoder).unwrap();

    assert_eq!(Decoded(42, 100500), actual);
}

#[test]
fn pass_struct() {
    let buf = [0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, RustcDecodable)]
    struct Decoded { id: u32, value: u32 };

    let mut decoder = Decoder::new(cur);
    let actual: Decoded = Decodable::decode(&mut decoder).unwrap();

    assert_eq!(Decoded { id: 42, value: 100500 }, actual);
}

#[test]
fn pass_option_some() {
    let buf = [0x1f];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: Option<u8> = Decodable::decode(&mut decoder).unwrap();
    assert_eq!(Some(31), actual);
}

#[test]
fn pass_option_some_null() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: Option<()> = Decodable::decode(&mut decoder).unwrap();
    assert_eq!(Some(()), actual);
}

#[test]
fn pass_option_none() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: Option<u8> = Decodable::decode(&mut decoder).unwrap();
    assert_eq!(None, actual);
}

#[test]
fn fail_option_u8_from_reserved() {
    let buf = [0xc1];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: Result<Option<u8>> = Decodable::decode(&mut decoder);
    match actual.err() {
        Some(Error::TypeMismatch(Marker::Reserved)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_vector() {
    let buf = [0x92, 0x00, 0xcc, 0x80];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: Vec<u8> = Decodable::decode(&mut decoder).unwrap();
    assert_eq!(vec![0, 128], actual);
}

#[test]
fn pass_map() {
    use std::collections::HashMap;

    let buf = [
        0x82, // 2 (size)
        0xa3, 0x69, 0x6e, 0x74, // 'int'
        0xcc, 0x80, // 128
        0xa3, 0x6b, 0x65, 0x79, // 'key'
        0x2a // 42
    ];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);
    let actual: HashMap<String, u8> = Decodable::decode(&mut decoder).unwrap();
    let mut expected = HashMap::new();
    expected.insert("int".to_string(), 128);
    expected.insert("key".to_string(), 42);

    assert_eq!(expected, actual);
}

#[test]
fn pass_enum() {
    let buf = [0x92, 0x01, 0x90];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, RustcDecodable)]
    enum Custom {
        First,
        Second,
    }

    let mut decoder = Decoder::new(cur);
    let actual: Custom = Decodable::decode(&mut decoder).unwrap();

    assert_eq!(Custom::Second, actual);
}

#[test]
fn pass_enum_variant_with_arg() {
    let buf = [0x92, 0x01, 0x91, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, RustcDecodable)]
    enum Custom {
        First,
        Second(u32),
    }

    let mut decoder = Decoder::new(cur);
    let actual: Custom = Decodable::decode(&mut decoder).unwrap();

    assert_eq!(Custom::Second(42), actual);
}

#[test]
fn fail_enum_sequence_mismatch() {
    let buf = [0x93, 0x1, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, RustcDecodable)]
    enum Custom {
        First,
        Second,
    }

    let mut decoder = Decoder::new(cur);
    let actual: Result<Custom> = Decodable::decode(&mut decoder);

    match actual.err() {
        Some(Error::Uncategorized(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn fail_enum_overflow() {
    let buf = [0x92, 0x01, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, RustcDecodable)]
    enum Custom {
        First,
    }

    let mut decoder = Decoder::new(cur);
    let actual: Result<Custom> = Decodable::decode(&mut decoder);

    match actual.err() {
        Some(Error::Uncategorized(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_struct_enum_with_arg() {
    let buf = [0x92, 0x01, 0x91, 0x2a];
    let cur = Cursor::new(&buf[..]);

    #[derive(Debug, PartialEq, RustcDecodable)]
    enum Custom {
        First,
        Second { id: u32 },
    }

    let mut decoder = Decoder::new(cur);
    let actual: Custom = Decodable::decode(&mut decoder).unwrap();

    assert_eq!(Custom::Second { id: 42 }, actual);
}

#[test]
fn pass_decoder_get_ref() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
    assert_eq!(1, decoder.get_ref().position());
}

#[test]
fn pass_decoder_get_mut() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
    decoder.get_mut().set_position(0);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_decoder_into_inner() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
    let cur = decoder.into_inner();

    assert_eq!(1, cur.position());
}
