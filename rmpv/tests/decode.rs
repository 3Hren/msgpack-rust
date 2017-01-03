extern crate rmpv;

use rmpv::Value;
use rmpv::decode::value::{read_value, Error};

#[test]
fn from_null_decode_value() {
    let buf = [0xc0];
    assert_eq!(Value::Nil, read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_pfix_decode_value() {
    let buf = [0x1f];
    assert_eq!(Value::U64(31), read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_bool_decode_value() {
    let buf = [0xc3, 0xc2];
    assert_eq!(Value::Boolean(true), read_value(&mut &buf[..]).unwrap());
    assert_eq!(Value::Boolean(false), read_value(&mut &buf[1..]).unwrap());
}

#[test]
fn from_i32_decode_value() {
    let buf = [0xd2, 0xff, 0xff, 0xff, 0xff];
    assert_eq!(Value::I64(-1), read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_f64_decode_value() {
    let buf = [0xcb, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    assert_eq!(Value::F64(::std::f64::NEG_INFINITY), read_value(&mut &buf[..]).unwrap());
}


#[test]
fn from_strfix_decode_value() {
    let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    assert_eq!(Value::String("le message".into()), read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_fixarray_decode_value() {
    let buf = [
        0x93,
        0x00, 0x2a, 0xf7
    ];

    let expected = Value::Array(vec![
        Value::U64(0),
        Value::U64(42),
        Value::I64(-9),
    ]);

    assert_eq!(expected, read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_fixarray_incomplete_decode_value() {
    let buf = [
        0x93,
        0x00, 0x2a
    ];

    match read_value(&mut &buf[..]) {
        Err(Error::InvalidMarkerRead(..)) => {}
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn from_fixmap_decode_value() {
    let buf = [
        0x82,
        0x2a,
        0xce, 0x0, 0x1, 0x88, 0x94,
        0xa3, 0x6b, 0x65, 0x79,
        0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65
    ];

    let expected = Value::Map(vec![
        (Value::U64(42), Value::U64(100500)),
        (Value::String("key".into()), Value::String("value".into())),
    ]);

    assert_eq!(expected, read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_fixext1_decode_value() {
    let buf = [0xd4, 0x01, 0x02];
    assert_eq!(Value::Ext(1, vec![2]), read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_str8_decode_value() {
    let buf: &[u8] = &[
        0xd9,
        0x20,
        0x42,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x45
    ];

    assert_eq!(Value::String("B123456789012345678901234567890E".into()),
        read_value(&mut &buf[..]).unwrap());
}

#[test]
fn from_str8_invalid_utf8() {
    // Invalid 2 Octet Sequence.
    let buf: &[u8] = &[0xd9, 0x02, 0xc3, 0x28];

    match read_value(&mut &buf[..]) {
        Err(Error::FromUtf8Error(err)) => {
            assert_eq!(buf[2..].to_vec(), err.into_bytes());
        }
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn from_array_of_two_integers() {
    let buf: &[u8] = &[0x92, 0x04, 0x2a];

    let vec = vec![Value::U64(4), Value::U64(42)];
    assert_eq!(Value::Array(vec), read_value(&mut &buf[..]).unwrap());
}
