use std::io::Cursor;

use rustc_serialize::Encodable;

use msgpack::Encoder;

#[test]
fn pass_null() {
    let mut buf = [0x00];

    let val = ();
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xc0], buf);
}

#[test]
fn pass_bool() {
    let mut buf = [0x00, 0x00];

    {
        let mut cur = Cursor::new(&mut buf[..]);

        let mut encoder = Encoder::new(&mut cur);

        let val = true;
        val.encode(&mut encoder).ok().unwrap();
        let val = false;
        val.encode(&mut encoder).ok().unwrap();
    }

    assert_eq!([0xc3, 0xc2], buf);
}

#[test]
fn pass_usize() {
    let mut buf = [0x00, 0x00];

    let val = 255usize;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcc, 0xff], buf);
}

#[test]
fn pass_u8() {
    let mut buf = [0x00, 0x00];

    let val = 255u8;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcc, 0xff], buf);
}

#[test]
fn pass_u16() {
    let mut buf = [0x00, 0x00, 0x00];

    let val = 65535u16;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcd, 0xff, 0xff], buf);
}

#[test]
fn pass_u32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 4294967295u32;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xce, 0xff, 0xff, 0xff, 0xff], buf);
}

#[test]
fn pass_u64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 18446744073709551615u64;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], buf);
}

#[test]
fn pass_i8() {
    let mut buf = [0x00, 0x00];

    let val = -128i8;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd0, 0x80], buf);
}

#[test]
fn pass_i64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = -9223372036854775808i64;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}
