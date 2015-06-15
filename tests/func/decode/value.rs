use std::io::Cursor;

use msgpack::decode::value::*;

#[test]
fn from_null_decode_value() {
    let buf = [0xc0, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(Value::Nil, read_value(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_pfix_decode_value() {
    let buf = [0x1f];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(Value::Integer(Integer::U64(31)), read_value(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i32_decode_value() {
    let buf = [0xd2, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(Value::Integer(Integer::I64(-1)), read_value(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_f64_decode_value() {
    use std::f64;

    let buf = [0xcb, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(Value::Float(Float::F64(f64::NEG_INFINITY)), read_value(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_strfix_decode_value() {
    let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(Value::String("le message".to_string()), read_value(&mut cur).unwrap());
    assert_eq!(11, cur.position());
}

#[test]
fn from_fixarray_decode_value() {
    let buf = [
        0x93,
        0x00, 0x2a, 0xf7
    ];
    let mut cur = Cursor::new(&buf[..]);

    let expected = Value::Array(vec![
        Value::Integer(Integer::U64(0)),
        Value::Integer(Integer::U64(42)),
        Value::Integer(Integer::I64(-9)),
    ]);

    assert_eq!(expected, read_value(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_fixarray_incomplete_decode_value() {
    let buf = [
        0x93,
        0x00, 0x2a
    ];
    let mut cur = Cursor::new(&buf[..]);

    match read_value(&mut cur) {
        Err(Error::InvalidArrayRead(err)) => {
            match *err {
                Error::InvalidMarkerRead(..) => (),
                other => panic!("unexpected result: {:?}", other)
            }
        }
        other => panic!("unexpected result: {:?}", other)
    }
    assert_eq!(3, cur.position());
}

#[test]
fn from_fixmap_decode_value() {
    let buf = [
        0x82, // size: 2
        0x2a, // 42
        0xce, 0x0, 0x1, 0x88, 0x94, // 100500
        0xa3, 0x6b, 0x65, 0x79, // 'key'
        0xa5, 0x76, 0x61, 0x6c, 0x75, 0x65 // 'value'
    ];
    let mut cur = Cursor::new(&buf[..]);

    let expected = Value::Map(vec![
        (Value::Integer(Integer::U64(42)), Value::Integer(Integer::U64(100500))),
        (Value::String("key".to_string()), Value::String("value".to_string())),
    ]);

    assert_eq!(expected, read_value(&mut cur).unwrap());
    assert_eq!(17, cur.position());
}

#[test]
fn from_fixext1_decode_value() {
    let buf = [0xd4, 0x01, 0x02];
    let mut cur = Cursor::new(&buf[..]);

    assert_eq!(Value::Ext(1, vec![2]), read_value(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_str8_decode_value() {
    let buf: &[u8] = &[
        0xd9, // Type.
        0x20, // Size
        0x42, // B
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
        0x45  // E
    ];
    let mut cur = Cursor::new(buf);

    assert_eq!(Value::String("B123456789012345678901234567890E".to_string()),
        read_value(&mut cur).unwrap());
    assert_eq!(34, cur.position());
}

//#[test]
//fn from_str8_with_unnecessary_bytes_decode_value() {
//    let buf: &[u8] = &[
//        0xd9, // Type.
//        0x20, // Size
//        0x42, // B
//        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
//        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30,
//        0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x30
//    ];
//    let mut cur = Cursor::new(buf);

//    assert_eq!(Error::InvalidDataCopy(buf[2..].to_vec(), ReadError::UnexpectedEOF),
//        read_value(&mut cur).err().unwrap());
//    assert_eq!(33, cur.position());
//}

#[test]
fn from_str8_invalid_utf8() {
    // Invalid 2 Octet Sequence.
    let buf: &[u8] = &[0xd9, 0x02, 0xc3, 0x28];
    let mut cur = Cursor::new(buf);

    match read_value(&mut cur) {
        Err(Error::InvalidUtf8(raw, _)) => {
            assert_eq!(buf[2..].to_vec(), raw);
        }
        other => panic!("unexpected result: {:?}", other)
    }

    assert_eq!(4, cur.position());
}

#[test]
fn from_array_of_two_integers() {
    let buf: &[u8] = &[0x92, 0x04, 0x2a];
    let mut cur = Cursor::new(buf);

    let vec = vec![Value::Integer(Integer::U64(4)), Value::Integer(Integer::U64(42))];
    assert_eq!(Value::Array(vec),
        read_value(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}
