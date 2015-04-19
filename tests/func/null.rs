use std::io::Cursor;

use msgpack::core::*;
use msgpack::core::decode::*;

#[test]
fn from_nil() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!((), read_nil(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nil_invalid_marker() {
    let buf: &[u8] = &[0xc1];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Reserved),
        read_nil(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_nil_invalid_marker_read() {
    let buf: &[u8] = &[];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarkerRead(ReadError::UnexpectedEOF),
        read_nil(&mut cur).err().unwrap());
    assert_eq!(0, cur.position());
}
