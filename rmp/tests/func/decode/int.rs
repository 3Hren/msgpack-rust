use std::io::Cursor;

use msgpack::Marker;
use msgpack::decode::*;

#[test]
fn from_unsigned_fixnum_read_uint() {
    let buf = [0x00, 0x7f, 0x20];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(0u64, read_uint(&mut cur).unwrap());
    assert_eq!(1, cur.position());

    assert_eq!(127u64, read_uint(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(32u64, read_uint(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_unsigned_u8_read_uint() {
    let buf = [0xcc, 0x80, 0xcc, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(128u64, read_uint(&mut cur).unwrap());
    assert_eq!(2, cur.position());

    assert_eq!(255u64, read_uint(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_unsigned_u8_incomplete_read_uint() {
    let buf = [0xcc];
    let mut cur = Cursor::new(&buf[..]);

    read_uint(&mut cur).err().unwrap();
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u16_read_uint() {
    let buf = [0xcd, 0x01, 0x00, 0xcd, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(256u64, read_uint(&mut cur).unwrap());
    assert_eq!(3, cur.position());

    assert_eq!(65535u64, read_uint(&mut cur).unwrap());
    assert_eq!(6, cur.position());
}

#[test]
fn from_unsigned_u16_incomplete_read_uint() {
    let buf = [0xcd];
    let mut cur = Cursor::new(&buf[..]);

    read_uint(&mut cur).err().unwrap();
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u32_read_uint() {
    let buf = [0xce, 0x00, 0x01, 0x00, 0x00, 0xce, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(65536u64, read_uint(&mut cur).unwrap());
    assert_eq!(5, cur.position());

    assert_eq!(4294967295u64, read_uint(&mut cur).unwrap());
    assert_eq!(10, cur.position());
}

#[test]
fn from_unsigned_u32_incomplete_read_uint() {
    let buf = [0xce];
    let mut cur = Cursor::new(&buf[..]);

    read_uint(&mut cur).err().unwrap();
    assert_eq!(1, cur.position());
}

#[test]
fn from_unsigned_u64_read_uint() {
    let buf = [
        0xcf, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
        0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
    ];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(4294967296u64, read_uint(&mut cur).unwrap());
    assert_eq!(9, cur.position());

    assert_eq!(18446744073709551615u64, read_uint(&mut cur).unwrap());
    assert_eq!(18, cur.position());
}

#[test]
fn from_unsigned_u64_incomplete_read_uint() {
    let buf = [0xcf];
    let mut cur = Cursor::new(&buf[..]);

    read_uint(&mut cur).err().unwrap();
    assert_eq!(1, cur.position());
}

// #[test]
// fn from_unsigned_invalid_marker_read_u64_loosely() {
//     let buf = [0xc0];
//     let mut cur = Cursor::new(&buf[..]);
//
//     match read_u64_loosely(&mut cur) {
//         Err(ValueReadError::TypeMismatch(Marker::Null)) => (),
//         other => panic!("unexpected result: {:?}", other)
//     }
//     assert_eq!(1, cur.position());
// }
//
// #[test]
// fn from_unsigned_invalid_unknown_marker_read_u64_loosely() {
//     let buf = [0xc1];
//     let mut cur = Cursor::new(&buf[..]);
//
//     match read_u64_loosely(&mut cur) {
//         Err(ValueReadError::TypeMismatch(Marker::Reserved)) => (),
//         other => panic!("unexpected result: {:?}", other)
//     }
//     assert_eq!(1, cur.position());
// }
//
// #[test]
// fn from_nfix_min_read_i64_loosely() {
//     let buf: &[u8] = &[0xe0];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(-32, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(1, cur.position());
// }
//
// #[test]
// fn from_nfix_max_read_i64_loosely() {
//     let buf: &[u8] = &[0xff];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(-1, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(1, cur.position());
// }
//
// #[test]
// fn from_i8_min_read_i64_loosely() {
//     let buf: &[u8] = &[0xd0, 0x80];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(-128, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(2, cur.position());
// }
//
// #[test]
// fn from_i8_max_read_i64_loosely() {
//     let buf: &[u8] = &[0xd0, 0x7f];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(127, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(2, cur.position());
// }
//
// #[test]
// fn from_i16_min_read_i64_loosely() {
//     let buf: &[u8] = &[0xd1, 0x80, 0x00];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(-32768, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(3, cur.position());
// }
//
// #[test]
// fn from_i16_max_read_i64_loosely() {
//     let buf: &[u8] = &[0xd1, 0x7f, 0xff];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(32767, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(3, cur.position());
// }
//
// #[test]
// fn from_i32_min_read_i64_loosely() {
//     let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(-2147483648, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(5, cur.position());
// }
//
// #[test]
// fn from_i32_max_read_i64_loosely() {
//     let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(2147483647, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(5, cur.position());
// }
//
// #[test]
// fn from_i64_min_read_i64_loosely() {
//     let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(-9223372036854775808, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(9, cur.position());
// }
//
// #[test]
// fn from_i64_max_read_i64_loosely() {
//     let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
//     let mut cur = Cursor::new(buf);
//
//     assert_eq!(9223372036854775807, read_i64_loosely(&mut cur).unwrap());
//     assert_eq!(9, cur.position());
// }
