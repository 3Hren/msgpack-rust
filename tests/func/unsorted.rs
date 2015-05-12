use std::io::Cursor;

use msgpack::core::*;
use msgpack::core::decode::*;

#[test]
fn from_null_read_len() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        decode::read_bin_len(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_bin16_max_read_len() {
    let buf: &[u8] = &[0xc5, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65535, decode::read_bin_len(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_bin32_max_read_len() {
    let buf: &[u8] = &[0xc6, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(4294967295, decode::read_bin_len(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_fixstr_min_read_str_len() {
    let buf: &[u8] = &[0xa0];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixstr_rnd_read_str_len() {
    let buf: &[u8] = &[0xaa];
    let mut cur = Cursor::new(buf);

    assert_eq!(10, read_str_len(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixstr_max_read_str_len() {
    let buf: &[u8] = &[0xbf];
    let mut cur = Cursor::new(buf);

    assert_eq!(31, read_str_len(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str8_min_read_str_len() {
    let buf: &[u8] = &[0xd9, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str8_rnd_read_str_len() {
    let buf: &[u8] = &[0xd9, 0x0a];
    let mut cur = Cursor::new(buf);

    assert_eq!(10, read_str_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str8_read_str_len_eof() {
    let buf: &[u8] = &[0xd9];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str8_max_read_str_len() {
    let buf: &[u8] = &[0xd9, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(255, read_str_len(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str16_min_read_str_len() {
    let buf: &[u8] = &[0xda, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_str16_max_read_str_len() {
    let buf: &[u8] = &[0xda, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65535, read_str_len(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_str16_read_str_len_eof() {
    let buf: &[u8] = &[0xda, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_str32_min_read_str_len() {
    let buf: &[u8] = &[0xdb, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_str_len(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_str32_max_read_str_len() {
    let buf: &[u8] = &[0xdb, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(4294967295, read_str_len(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_str32_read_str_len_eof() {
    let buf: &[u8] = &[0xdb, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_null_read_str_len() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_str_len(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str_strfix() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 16];

    assert_eq!("le message", read_str(&mut cur, &mut out).unwrap());
    assert_eq!(11, cur.position());
}

#[test]
fn from_str_strfix_extra_data() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x00];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 16];

    assert_eq!("le message", read_str(&mut cur, &mut out).unwrap());
    assert_eq!(11, cur.position());
}

#[test]
fn from_str_strfix_exact_buffer() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 10];

    assert_eq!("le message", read_str(&mut cur, &mut out).unwrap());
    assert_eq!(11, cur.position());
}

#[test]
fn from_str_strfix_insufficient_bytes() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 16];

    assert_eq!(DecodeStringError::InvalidDataCopy(&[0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67], ReadError::UnexpectedEOF),
        read_str(&mut cur, &mut out).err().unwrap());
    assert_eq!(10, cur.position());
}

#[test]
fn from_str_strfix_invalid_utf8() {
    // Invalid 2 Octet Sequence.
    let buf: &[u8] = &[0xa2, 0xc3, 0x28];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 16];

    match read_str(&mut cur, &mut out) {
        Err(DecodeStringError::InvalidUtf8(raw, _)) => {
            assert_eq!(&[0xc3, 0x28], raw);
        }
        other => panic!("unexpected result: {:?}", other)
    }

    assert_eq!(3, cur.position());
}

#[test]
fn from_str_strfix_buffer_too_small() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
    let mut cur = Cursor::new(buf);

    let mut out: &mut [u8] = &mut [0u8; 9];

    assert_eq!(DecodeStringError::BufferSizeTooSmall(10),
        read_str(&mut cur, &mut out).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_str_strfix_ref() {
    let buf: &[u8] = &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];

    let out = read_str_ref(&buf).unwrap();

    assert_eq!(10, out.len());
    assert!(buf[1..11] == out[0..10])
}

#[test]
fn from_i8_min() {
    let buf: &[u8] = &[0xd0, 0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(-128, read_i8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_max() {
    let buf: &[u8] = &[0xd0, 0x7f];
    let mut cur = Cursor::new(buf);

    assert_eq!(127, read_i8(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i8_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_i8(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i8_unexpected_eof() {
    let buf: &[u8] = &[0xd0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_i8(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_u16_min() {
    let buf: &[u8] = &[0xcd, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_u16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_u32_max() {
    let buf: &[u8] = &[0xce, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(4294967295, read_u32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i16_min() {
    let buf: &[u8] = &[0xd1, 0x80, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-32768, read_i16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_max() {
    let buf: &[u8] = &[0xd1, 0x7f, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(32767, read_i16(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_i16_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_i16(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i16_unexpected_eof() {
    let buf: &[u8] = &[0xd1, 0x7f];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_i16(&mut cur).err().unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_i32_min() {
    let buf: &[u8] = &[0xd2, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-2147483648, read_i32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_max() {
    let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(2147483647, read_i32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_i32_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_i32(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i32_unexpected_eof() {
    let buf: &[u8] = &[0xd2, 0x7f, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_i32(&mut cur).err().unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_i64_min() {
    let buf: &[u8] = &[0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-9223372036854775808, read_i64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_max() {
    let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(9223372036854775807, read_i64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_i64_type_mismatch() {
    let buf: &[u8] = &[0xc0, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_i64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_i64_unexpected_eof() {
    let buf: &[u8] = &[0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_i64(&mut cur).err().unwrap());
    assert_eq!(8, cur.position());
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

#[test]
fn from_empty_array_read_size() {
    let buf: &[u8] = &[0x90];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_array_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixarray_max_read_size() {
    let buf: &[u8] = &[
        0x9f,
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e
    ];
    let mut cur = Cursor::new(buf);

    assert_eq!(15, read_array_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_array16_min_read_size() {
    let buf: &[u8] = &[0xdc, 0x00, 0x10];
    let mut cur = Cursor::new(buf);

    assert_eq!(16, read_array_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_array16_max_read_size() {
    let buf: &[u8] = &[0xdc, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65535, read_array_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_array16_unexpected_eof_read_size() {
    let buf: &[u8] = &[0xdc, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_array_size(&mut cur).err().unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_array32_min_read_size() {
    let buf: &[u8] = &[0xdd, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_array_size(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_array32_max_read_size() {
    let buf: &[u8] = &[0xdd, 0xff, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(4294967295, read_array_size(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_array32_unexpected_eof_read_size() {
    let buf: &[u8] = &[0xdd, 0xff, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_array_size(&mut cur).err().unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_null_read_array_size() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_array_size(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixmap_min_read_size() {
    let buf: &[u8] = &[0x80];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_map_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixmap_max_read_size() {
    let buf: &[u8] = &[0x8f];
    let mut cur = Cursor::new(buf);

    assert_eq!(15, read_map_size(&mut cur).unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_map16_min_read_size() {
    let buf: &[u8] = &[0xde, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_map_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_map16_max_read_size() {
    let buf: &[u8] = &[0xde, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(65535, read_map_size(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_map32_min_read_size() {
    let buf: &[u8] = &[0xdf, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0, read_map_size(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_null_read_map_size() {
    let buf: &[u8] = &[0xc0, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_map_size(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_f32_zero_plus() {
    let buf: &[u8] = &[0xca, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0.0, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_f32_max() {
    let buf: &[u8] = &[0xca, 0x7f, 0x7f, 0xff, 0xff];
    let mut cur = Cursor::new(buf);

    assert_eq!(3.4028234e38_f32, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_f32_inf() {
    use std::f32;

    let buf: &[u8] = &[0xca, 0x7f, 0x80, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f32::INFINITY, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_f32_neg_inf() {
    use std::f32;

    let buf: &[u8] = &[0xca, 0xff, 0x80, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f32::NEG_INFINITY, read_f32(&mut cur).unwrap());
    assert_eq!(5, cur.position());
}

#[test]
fn from_null_read_f32() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_f32(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_f64_zero_plus() {
    let buf: &[u8] = &[0xcb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(0.0, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_f64_zero_minus() {
    let buf: &[u8] = &[0xcb, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(-0.0, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_f64_inf() {
    use std::f64;

    let buf: &[u8] = &[0xcb, 0x7f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f64::INFINITY, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_f64_neg_inf() {
    use std::f64;

    let buf: &[u8] = &[0xcb, 0xff, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let mut cur = Cursor::new(buf);

    assert_eq!(f64::NEG_INFINITY, read_f64(&mut cur).unwrap());
    assert_eq!(9, cur.position());
}

#[test]
fn from_null_read_f64() {
    let buf: &[u8] = &[0xc0];
    let mut cur = Cursor::new(buf);

    assert_eq!(Error::TypeMismatch(Marker::Null),
        read_f64(&mut cur).err().unwrap());
    assert_eq!(1, cur.position());
}

#[test]
fn from_fixext1_read_fixext1() {
    let buf: &[u8] = &[0xd4, 0x01, 0x02];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, 2), read_fixext1(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_fixext2_read_fixext2() {
    let buf: &[u8] = &[0xd5, 0x01, 0x00, 0x02];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, 2), read_fixext2(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_fixext4_read_fixext4() {
    let buf: &[u8] = &[0xd6, 0x01, 0x00, 0x00, 0x00, 0x02];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, [0x00, 0x00, 0x00, 0x02]), read_fixext4(&mut cur).unwrap());
    assert_eq!(6, cur.position());
}

#[test]
fn from_fixext8_read_fixext8() {
    let buf: &[u8] = &[0xd7, 0x01, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
               read_fixext8(&mut cur).unwrap());
    assert_eq!(10, cur.position());
}

#[test]
fn from_fixext16_read_fixext16() {
    let buf: &[u8] = &[0xd8, 0x01,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
    let mut cur = Cursor::new(buf);

    assert_eq!((1, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]),
               read_fixext16(&mut cur).unwrap());
    assert_eq!(18, cur.position());
}

#[test]
fn from_fixext1_read_ext_meta() {
    let buf: &[u8] = &[0xd4, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 1 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext2_read_ext_meta() {
    let buf: &[u8] = &[0xd5, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 2 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext4_read_ext_meta() {
    let buf: &[u8] = &[0xd6, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 4 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext8_read_ext_meta() {
    let buf: &[u8] = &[0xd7, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 8 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_fixext16_read_ext_meta() {
    let buf: &[u8] = &[0xd8, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 16 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(2, cur.position());
}

#[test]
fn from_ext8_read_ext_meta() {
    let buf: &[u8] = &[0xc7, 0xff, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 255 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(3, cur.position());
}

#[test]
fn from_ext16_read_ext_meta() {
    let buf: &[u8] = &[0xc8, 0xff, 0xff, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 65535 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(4, cur.position());
}

#[test]
fn from_ext32_read_ext_meta() {
    let buf: &[u8] = &[0xc9, 0xff, 0xff, 0xff, 0xff, 0x01];
    let mut cur = Cursor::new(buf);

    assert_eq!(ExtMeta { typeid: 1, size: 4294967295 }, read_ext_meta(&mut cur).unwrap());
    assert_eq!(6, cur.position());
}

#[test]
fn from_bin8_read_zero_copy() {
    let buf = [0xc4, 0x05, 0x00, 0x00, 0x2a, 0x00, 0x00];

    assert_eq!([0x00, 0x00, 0x2a, 0x00, 0x00], read_bin_borrow(&mut &buf[..]).unwrap());
}

#[test]
fn from_bin8_read_zero_copy_insufficient_bytes() {
    let buf = [0xc4, 0x05, 0x00, 0x00, 0x2a, 0x00];

    assert_eq!(Error::InvalidDataRead(ReadError::UnexpectedEOF),
        read_bin_borrow(&mut &buf[..]).err().unwrap());
}
