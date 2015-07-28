use msgpack::ValueRef;
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
