use std::io;
use std::io::Cursor;

use msgpack::core::*;
use msgpack::core::decode::*;
use msgpack::core::encode::*;

#[test]
fn pass_read_nil() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!((), read_nil(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn fail_read_nil_invalid_marker() {
    let buf: &[u8] = &[0xc1];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Reserved),
        read_nil(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn fail_read_nil_invalid_marker_read() {
    let buf: &[u8] = &[];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidMarkerRead(ReadError::UnexpectedEOF),
        read_nil(&mut cur).err().unwrap());
    assert_eq!(0, cur.position());
}

#[test]
fn pass_pack() {
    let mut buf = [0x00];

    assert_eq!(1, write_nil(&mut &mut buf[..]).unwrap());
    assert_eq!([0xc0], buf);
}

#[test]
fn fail_pack_too_small_buffer() {
    let mut buf = [];

    assert_eq!(Error::InvalidMarkerWrite(WriteError::IO(io::Error::new(io::ErrorKind::WriteZero, ""))),
        write_nil(&mut &mut buf[..]).err().unwrap());
}
