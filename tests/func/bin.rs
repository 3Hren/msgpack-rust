use std::io::Cursor;

use msgpack::core::*;
use msgpack::core::decode::*;

#[test]
fn from_bin8_min_read_len() {
    let buf: &[u8] = &[0xc4, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, decode::read_bin_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_bin8_max_read_len() {
    let buf: &[u8] = &[0xc4, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(255, decode::read_bin_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_bin8_eof_read_len() {
    let buf: &[u8] = &[0xc4];
    let mut cur = Cursor::new(buf);

    assert_err!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        decode::read_bin_len(&mut cur));
    assert_eq!(1, cur.position());
}
