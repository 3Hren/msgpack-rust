use std::io::Cursor;

use msgpack::decode::*;
//use msgpack::decode::FixedValueReadError;

#[test]
fn pass_read_nil() {
    let buf = [0xc0];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!((), read_nil(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn fail_read_nil_invalid_marker() {
    let buf = [0xc1];
    let mut cur = Cursor::new(&buf[..]);

    match read_nil(&mut cur) {
        Err(FixedValueReadError::TypeMismatch(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(1, cur.position());
}

#[test]
fn fail_read_nil_invalid_marker_read() {
    let buf = [];
    let mut cur = Cursor::new(&buf[..]);

    match read_nil(&mut cur) {
        Err(FixedValueReadError::InvalidMarkerRead(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(0, cur.position());
}
