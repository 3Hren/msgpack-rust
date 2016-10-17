use std::io::Cursor;

use msgpack::Marker;
use msgpack::decode::*;

#[test]
fn from_positive_fixnum() {
    let buf = [0x00, 0x7f, 0x20];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(0u8, read_pfix(&mut cur).unwrap());
    assert_eq!(1, cur.position());

    assert_eq!(127u8, read_pfix(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(32u8, read_pfix(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_u8_min() {
    let buf = [0xcc, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(0, read_u8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_u8_max() {
    let buf = [0xcc, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(255, read_u8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_u8_type_mismatch() {
    let buf = [0xc0, 0x80];
    let mut cur = Cursor::new(&buf[..]);

    match read_u8(&mut cur) {
        Err(ValueReadError::TypeMismatch(Marker::Null)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_u8_unexpected_eof() {
    let buf = [0xcc];
    let mut cur = Cursor::new(&buf[..]);

    read_u8(&mut cur).err().unwrap();
    assert_eq!(1, cur.position());
}

#[test]
fn from_u16_min() {
    let buf = [0xcd, 0x00, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(0, read_u16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_u32_max() {
    let buf = [0xce, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(4294967295, read_u32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}
