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
fn from_nfix_min() {
    let buf = [0xe0];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(-32, read_nfix(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nfix_max() {
    let buf = [0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(-1, read_nfix(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nfix_type_mismatch() {
    let buf = &[0xc0];
    let mut cur = Cursor::new(&buf[..]);

    match read_nfix(&mut cur) {
        Err(FixedValueReadError::TypeMismatch(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
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

    match read_u8(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
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

#[test]
fn from_i8_min() {
    let buf = [0xd0, 0x80];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(-128, read_i8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_max() {
    let buf = [0xd0, 0x7f];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(127, read_i8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_type_mismatch() {
    let buf = [0xc0, 0x80];
    let mut cur = Cursor::new(&buf[..]);

    match read_i8(&mut cur) {
        Err(ValueReadError::TypeMismatch(Marker::Null)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_i8_unexpected_eof() {
    let buf = [0xd0];
    let mut cur = Cursor::new(&buf[..]);

    match read_i8(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_i16_min() {
    let buf = [0xd1, 0x80, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(-32768, read_i16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_max() {
    let buf = [0xd1, 0x7f, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(32767, read_i16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_type_mismatch() {
    let buf = [0xc0, 0x80, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    match read_i16(&mut cur) {
        Err(ValueReadError::TypeMismatch(Marker::Null)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_i16_unexpected_eof() {
    let buf = [0xd1, 0x7f];
    let mut cur = Cursor::new(&buf[..]);

    match read_i16(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(2, cur.position());
}

#[test]
fn from_i32_min() {
    let buf = [0xd2, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(-2147483648, read_i32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_max() {
    let buf = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(2147483647, read_i32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_type_mismatch() {
    let buf = &[0xc0, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    match read_i32(&mut cur) {
        Err(ValueReadError::TypeMismatch(Marker::Null)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_i32_unexpected_eof() {
    let buf = &[0xd2, 0x7f, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    match read_i32(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(4, cur.position());
}

#[test]
fn from_i64_min() {
    let buf = [0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(-9223372036854775808, read_i64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_max() {
    let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(9223372036854775807, read_i64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_type_mismatch() {
    let buf = [0xc0, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    match read_i64(&mut cur) {
        Err(ValueReadError::TypeMismatch(Marker::Null)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_i64_unexpected_eof() {
    let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    match read_i64(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(8, cur.position());
}

#[test]
fn from_unsigned_fixnum_read_u64_loosely() {
    let buf = [0x00, 0x7f, 0x20];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(0u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(1, cur.position());

    assert_eq!(127u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(32u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_unsigned_u8_read_u64_loosely() {
    let buf = [0xcc, 0x80, 0xcc, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(128u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(255u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_unsigned_u8_incomplete_read_u64_loosely() {
    let buf = [0xcc];
    let mut cur = Cursor::new(&buf[..]);

    match read_u64_loosely(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u16_read_u64_loosely() {
    let buf = [0xcd, 0x01, 0x00, 0xcd, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(256u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(3, cur.position());

    assert_eq!(65535u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(6, cur.position());
}

#[test]
fn from_unsigned_u16_incomplete_read_u64_loosely() {
    let buf = [0xcd];
    let mut cur = Cursor::new(&buf[..]);

    match read_u64_loosely(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u32_read_u64_loosely() {
    let buf = [0xce, 0x00, 0x01, 0x00, 0x00, 0xce, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(65536u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(5, cur.position());

    assert_eq!(4294967295u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(10, cur.position());
}

#[test]
fn from_unsigned_u32_incomplete_read_u64_loosely() {
    let buf = [0xce];
    let mut cur = Cursor::new(&buf[..]);

    match read_u64_loosely(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u64_read_u64_loosely() {
    let buf = [
        0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
    ];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(4294967296u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(9, cur.position());

    assert_eq!(18446744073709551615u64, read_u64_loosely(&mut cur).unwrap());
    assert_eq!(18, cur.position());
}

#[test]
fn from_unsigned_u64_incomplete_read_u64_loosely() {
    let buf = [0xcf];
    let mut cur = Cursor::new(&buf[..]);

    match read_u64_loosely(&mut cur) {
        Err(ValueReadError::InvalidDataRead(ReadError::UnexpectedEOF)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_invalid_marker_read_u64_loosely() {
    let buf = [0xc0];
    let mut cur = Cursor::new(&buf[..]);

    match read_u64_loosely(&mut cur) {
        Err(ValueReadError::TypeMismatch(Marker::Null)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_invalid_unknown_marker_read_u64_loosely() {
    let buf = [0xc1];
    let mut cur = Cursor::new(&buf[..]);

    match read_u64_loosely(&mut cur) {
        Err(ValueReadError::TypeMismatch(Marker::Reserved)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn from_nfix_min_read_i64_loosely() {
    let buf: &[u8] = &[0xe0];
    let mut cur = Cursor::new(buf);

    assert_eq!(-32, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nfix_max_read_i64_loosely() {
    let buf: &[u8] = &[0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(-1, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i8_min_read_i64_loosely() {
    let buf: &[u8] = &[0xd0, 0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(-128, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_max_read_i64_loosely() {
    let buf: &[u8] = &[0xd0, 0x7f];
    let mut cur = Cursor::new(buf);

    assert_eq!(127, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i16_min_read_i64_loosely() {
    let buf: &[u8] = &[0xd1, 0x80, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-32768, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_max_read_i64_loosely() {
    let buf: &[u8] = &[0xd1, 0x7f, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(32767, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i32_min_read_i64_loosely() {
    let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-2147483648, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_max_read_i64_loosely() {
    let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(2147483647, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i64_min_read_i64_loosely() {
    let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-9223372036854775808, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_max_read_i64_loosely() {
    let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(9223372036854775807, read_i64_loosely(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}
