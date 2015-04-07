use std::io::Cursor;
use std::str::Utf8Error;

use msgpack::core::{Value, Integer, ReadError};
use msgpack::core::decode::value::*;

#[test]
fn from_i32_decode_value() {
    let buf: &[u8] = &[0xd2, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Value::Integer(Integer::I64(-1)), read_value(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_str8_decode_value() {
    let buf: &[u8] = &[
        0xd9, // Type.
        0x20, // Size
        0x42, // B
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x45  // E
    ];
    let mut cur = Cursor::new(buf);

    assert_eq!(Value::String("B123456789012345678901234567890E".to_string()),
        read_value(&mut cur).unwrap());
    assert_eq!(34, cur.position());
}

#[test]
fn from_str8_with_unnecessary_bytes_decode_value() {
    let buf: &[u8] = &[
        0xd9, // Type.
        0x20, // Size
        0x42, // B
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30
    ];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataCopy(buf[2..].to_vec(), ReadError::UnexpectedEOF),
        read_value(&mut cur).err().unwrap());
    assert_eq!(33, cur.position());
}

#[test]
fn from_str8_invalid_utf8() {
    // Invalid 2 Octet Sequence.
    let buf: &[u8] = &[0xd9, 0x02, 0xc3, 0x28];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidUtf8(buf[2..].to_vec(), Utf8Error::InvalidByte(0x0)),
        read_value(&mut cur).err().unwrap());
    assert_eq!(4, cur.position());
}

// TODO: decode_value_ref(&'a [u8]) -> &'a ValueRef<'a>
