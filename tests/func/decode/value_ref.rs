use msgpack::ValueRef;
use msgpack::decode::read_value_ref;
use msgpack::decode::value_ref::Error;

#[test]
fn from_strfix() {
    let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let mut rd = &buf[..];

    assert_eq!(ValueRef::String("le message"), read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_str8() {
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
fn from_str16() {
    let buf = [
        0xda, // Type.
        0x00, 0x20, // Size
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
fn from_str32() {
    let buf = [
        0xdb, // Type.
        0x00, 0x00, 0x00, 0x20, // Size
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

#[test]
fn from_string_insufficient_bytes_while_reading_length() {
    let buf = [0xd9];
    let mut rd = &buf[..];

    match read_value_ref(&mut rd).err().unwrap() {
        Error::InvalidLengthRead(..) => (),
        _ => panic!(),
    }
}

#[test]
fn from_string_insufficient_bytes_while_reading_data() {
    let buf = [
        0xd9, // Type.
        0x20, // Size == 32
        0x42, // B
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30
    ];

    let mut rd = &buf[..];

    match read_value_ref(&mut rd).err().unwrap() {
        Error::InvalidDataRead(..) => (),
        _ => panic!(),
    }
}

#[test]
fn from_string_invalid_utf8() {
    // Invalid 2 Octet Sequence.
    let buf = [0xd9, 0x02, 0xc3, 0x28];

    let mut rd = &buf[..];

    match read_value_ref(&mut rd).err().unwrap() {
        Error::InvalidUtf8(act, _) => { assert_eq!(&[0xc3, 0x28], act); },
        _ => panic!(),
    }
}

#[test]
fn from_bin8() {
    let buf = [0xc4, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Binary(&[0, 1, 2, 3, 4]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_bin16() {
    let buf = [0xc5, 0x00, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Binary(&[0, 1, 2, 3, 4]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_bin32() {
    let buf = [0xc6, 0x00, 0x00, 0x00, 0x05, 0x00, 0x01, 0x02, 0x03, 0x04];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Binary(&[0, 1, 2, 3, 4]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_bin8_eof_while_reading_data() {
    let buf = [0xc4, 0x05, 0x00, 0x01, 0x02, 0x03];

    let mut rd = &buf[..];

    match read_value_ref(&mut rd).err().unwrap() {
        Error::InvalidDataRead(..) => (),
        _ => panic!(),
    }
}

#[test]
fn from_fixext1() {
    let buf = [0xd4, 0x2a, 0xff];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_ext1_eof_while_reading_type() {
    let buf = [0xd4];

    let mut rd = &buf[..];

    match read_value_ref(&mut rd).err().unwrap() {
        Error::InvalidExtTypeRead(..) => (),
        _ => panic!(),
    }
}

#[test]
fn from_fixext2() {
    let buf = [0xd5, 0x2a, 0xff, 0xee];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255, 238]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_fixext4() {
    let buf = [0xd6, 0x2a, 0xff, 0xee, 0xdd, 0xcc];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255, 238, 221, 204]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_fixext8() {
    let buf = [0xd7, 0x2a, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255, 238, 221, 204, 187, 170, 153, 136]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_fixext16() {
    let buf = [
        0xd8, 0x2a,
        0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88,
        0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00
    ];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255, 238, 221, 204, 187, 170, 153, 136, 119, 102, 85, 68, 51, 34, 17, 0]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_ext8() {
    let buf = [0xc7, 0x04, 0x2a, 0xff, 0xee, 0xdd, 0xcc];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255, 238, 221, 204]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_ext16() {
    let buf = [0xc8, 0x00, 0x04, 0x2a, 0xff, 0xee, 0xdd, 0xcc];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255, 238, 221, 204]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_ext32() {
    let buf = [0xc9, 0x00, 0x00, 0x00, 0x04, 0x2a, 0xff, 0xee, 0xdd, 0xcc];

    let mut rd = &buf[..];

    assert_eq!(ValueRef::Ext(42, &[255, 238, 221, 204]),
        read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_fixmap() {
    let buf = [
        0x81, // size: 2
        // 0x2a, // 42
        // 0xce, 0x0, 0x1, 0x88, 0x94, // 100500
        0xa3, 0x6b, 0x65, 0x79, // 'key'
        0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65 // 'value'
    ];
    let mut rd = &buf[..];

    let map = vec![
        // (ValueRef::Integer(Integer::U64(42)), ValueRef::Integer(Integer::U64(100500))), TODO: Uncomment.
        (ValueRef::String("key"), ValueRef::String("value")),
    ];
    let expected = ValueRef::Map(map);

    assert_eq!(expected, read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_map16() {
    let buf = [
        0xde,
        0x00, 0x01,
        0xa3, 0x6b, 0x65, 0x79, // 'key'
        0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65 // 'value'
    ];
    let mut rd = &buf[..];

    let map = vec![(ValueRef::String("key"), ValueRef::String("value"))];
    let expected = ValueRef::Map(map);

    assert_eq!(expected, read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_map32() {
    let buf = [
        0xdf,
        0x00, 0x00, 0x00, 0x01,
        0xa3, 0x6b, 0x65, 0x79, // 'key'
        0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65 // 'value'
    ];
    let mut rd = &buf[..];

    let map = vec![(ValueRef::String("key"), ValueRef::String("value"))];
    let expected = ValueRef::Map(map);

    assert_eq!(expected, read_value_ref(&mut rd).ok().unwrap());
}

#[test]
fn from_fixarray() {
    let buf = [
        0x92,
        0xa2, 0x76, 0x31,
        0xa2, 0x76, 0x32,
    ];
    let mut rd = &buf[..];

    let vec = vec![ValueRef::String("v1"), ValueRef::String("v2")];

    assert_eq!(ValueRef::Array(vec), read_value_ref(&mut rd).unwrap());
}

#[test]
fn from_array16() {
    let buf = [
        0xdc,
        0x00, 0x02,
        0xa2, 0x76, 0x31,
        0xa2, 0x76, 0x32,
    ];
    let mut rd = &buf[..];

    let vec = vec![ValueRef::String("v1"), ValueRef::String("v2")];

    assert_eq!(ValueRef::Array(vec), read_value_ref(&mut rd).unwrap());
}

#[test]
fn from_array32() {
    let buf = [
        0xdd,
        0x00, 0x00, 0x00, 0x02,
        0xa2, 0x76, 0x31,
        0xa2, 0x76, 0x32,
    ];
    let mut rd = &buf[..];

    let vec = vec![ValueRef::String("v1"), ValueRef::String("v2")];

    assert_eq!(ValueRef::Array(vec), read_value_ref(&mut rd).unwrap());
}

// TODO: ValueRef with all possible types.
// TODO: Real-life examples.
