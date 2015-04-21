use std::io;

use msgpack::encode::Error;
use msgpack::encode::write_nil;

#[test]
fn pass_pack() {
    let mut buf = [0x00];

    write_nil(&mut &mut buf[..]).unwrap();

    assert_eq!([0xc0], buf);
}

#[test]
fn fail_pack_too_small_buffer() {
    let mut buf = [];

    match write_nil(&mut &mut buf[..]) {
        Err(Error::InvalidMarkerWrite(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}
