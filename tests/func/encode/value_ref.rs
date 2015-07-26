use msgpack::ValueRef;
use msgpack::encode::value_ref::write_value_ref;

#[test]
fn pack_nil() {
    let mut buf = [0x00];

    let val = ValueRef::Nil;

    write_value_ref(&mut &mut buf[..], &val).unwrap();

    assert_eq!([0xc0], buf);
}
