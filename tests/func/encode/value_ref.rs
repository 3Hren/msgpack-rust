use msgpack::ValueRef;
use msgpack::value::{Float, Integer};
use msgpack::encode::value_ref::write_value_ref;

#[test]
fn pack_nil() {
    let mut buf = [0x00];

    let val = ValueRef::Nil;

    write_value_ref(&mut &mut buf[..], &val).unwrap();

    assert_eq!([0xc0], buf);
}

#[test]
fn pack_nil_when_buffer_is_tool_small() {
    let mut buf = [];

    let val = ValueRef::Nil;

    match write_value_ref(&mut &mut buf[..], &val) {
        Err(..) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_pack_true() {
    let mut buf = [0x00];

    let val = ValueRef::Boolean(true);

    write_value_ref(&mut &mut buf[..], &val).unwrap();

    assert_eq!([0xc3], buf);
}

#[test]
fn pass_pack_uint_u16() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = ValueRef::Integer(Integer::U64(65535));

    write_value_ref(&mut &mut buf[..], &val).unwrap();

    assert_eq!([0xcd, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}

#[test]
fn pass_pack_i64() {
    let mut buf = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    let val = ValueRef::Integer(Integer::I64(-9223372036854775808));

    write_value_ref(&mut &mut buf[..], &val).unwrap();

    assert_eq!([0xd3, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], buf);
}

fn check_packed_eq(expected: &Vec<u8>, actual: &ValueRef) {
    let mut buf = Vec::new();

    write_value_ref(&mut buf, actual).unwrap();

    assert_eq!(*expected, buf);
}

#[test]
fn pass_pack_f32() {
    check_packed_eq(
        &vec![0xca, 0x7f, 0x7f, 0xff, 0xff],
        &ValueRef::Float(Float::F32(3.4028234e38_f32)));
}

#[test]
fn pass_pack_f64() {
    use std::f64;
    check_packed_eq(
        &vec![0xcb, 0x7f, 0xf0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        &ValueRef::Float(Float::F64(f64::INFINITY)));
}
