use msgpack::core::encode::*;

#[test]
fn pass_pack_pfix() {
    let mut buf = [0x00];

    write_pfix(&mut &mut buf[..], 127).ok().unwrap();

    assert_eq!([0x7f], buf);
}

#[test]
fn fail_pack_pfix_too_small_buffer() {
    let mut buf = [];

    match write_pfix(&mut &mut buf[..], 127) {
        Err(Error::InvalidFixedValueWrite(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn fail_pack_pfix_too_large() {
    let mut buf = [0x00];

    match write_pfix(&mut &mut buf[..], 128) {
        Err(Error::TypeMismatch) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_pack_u8() {
    let mut buf = [0x00, 0x00];

    write_u8(&mut &mut buf[..], 127).ok().unwrap();

    assert_eq!([0xcc, 0x7f], buf);
}

#[test]
fn pass_pack_u16() {
    let mut buf = [0x00, 0x00, 0x00];

    write_u16(&mut &mut buf[..], 65535).ok().unwrap();

    assert_eq!([0xcd, 0xff, 0xff], buf);
}

#[test]
fn pass_pack_u32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    write_u32(&mut &mut buf[..], 4294967295).ok().unwrap();

    assert_eq!([0xce, 0xff, 0xff, 0xff, 0xff], buf);
}

#[test]
fn pass_pack_u64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    write_u64(&mut &mut buf[..], 18446744073709551615).ok().unwrap();

    assert_eq!([0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff], buf);
}

#[test]
fn pass_pack_i8() {
    let mut buf = [0x00, 0x00];

    write_i8(&mut &mut buf[..], -128).ok().unwrap();

    assert_eq!([0xd0, 0x80], buf);
}

#[test]
fn pass_pack_i16() {
    let mut buf = [0x00, 0x00, 0x00];

    write_i16(&mut &mut buf[..], -32768).ok().unwrap();

    assert_eq!([0xd1, 0x80, 0x00], buf);
}

#[test]
fn pass_pack_i32() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00];

    write_i32(&mut &mut buf[..], -2147483648).ok().unwrap();

    assert_eq!([0xd2, 0x80, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_pack_i64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    write_i64(&mut &mut buf[..], -9223372036854775808).ok().unwrap();

    assert_eq!([0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}
