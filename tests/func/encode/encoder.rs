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
fn pass_isize() {
    let mut buf = [0x00, 0x00];

    let val = -128isize;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd0, 0x80], buf);
}

#[test]
fn pass_i8() {
    let mut buf = [0x00, 0x00];

    let val = -128i8;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd0, 0x80], buf);
}

#[test]
fn pass_i16() {
    let mut buf = [0x00, 0x00, 0x00];

    let val = -32768i16;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd1, 0x80, 0x00], buf);
}

#[test]
fn pass_i32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    let val = -2147483648i32;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd2, 0x80, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_i64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = -9223372036854775808i64;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_f32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 3.4028234e38_f32;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xca, 0x7f, 0x7f, 0xff, 0xff], buf);
}

#[test]
fn pass_f64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = 42f64;
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_char() {
    let mut buf = [0x00, 0x00];

    let val = '!';
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xa1, 0x21], buf);
}


#[test]
fn pass_string() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = "le message";
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65], buf);
}

#[test]
fn pass_struct() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    #[derive(RustcEncodable)]
    struct Decoded { id: u32, value: u32 }

    let val = Decoded { id: 42, value: 100500 };
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94], buf);
}
