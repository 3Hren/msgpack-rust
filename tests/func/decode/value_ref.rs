use msgpack::ValueRef;
use msgpack::decode::read_value_ref;
use msgpack::decode::value_ref::Error;

#[test]
fn from_string() {
    let buf = [
        0xd9, // Type.
        0x20, // Size
        0x42, // B
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x45  // E
    ];

    let mut slice = &buf[..];

    assert_eq!(ValueRef::String("B123456789012345678901234567890E"),
        read_value_ref(&mut slice).ok().unwrap());
}

#[test]
fn from_empty_buffer_invalid_marker_read() {
    let buf = [];

    let mut slice = &buf[..];

    match read_value_ref(&mut slice).err().unwrap() {
        Error::InvalidMarkerRead(..) => (),
        _ => panic!(),
    }
}

#[test]
fn from_empty_buffer_invalid_buffer_fill_because_eintr() {
    use std::io::{self, Read, BufRead};

    struct InterruptRead;

    impl Read for InterruptRead {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Interrupted, ""))
        }
    }

    impl BufRead for InterruptRead {
        fn fill_buf(&mut self) -> io::Result<&[u8]> { Err(io::Error::new(io::ErrorKind::Interrupted, "")) }
        fn consume(&mut self, _n: usize) {}
    }

    let mut rd = InterruptRead;

    match read_value_ref(&mut rd).err().unwrap() {
        Error::InvalidBufferFill(..) => (),
        _ => panic!(),
    }
}
