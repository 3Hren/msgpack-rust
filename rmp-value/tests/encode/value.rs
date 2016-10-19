use msgpack::Value;
use msgpack::encode::value::write_value;

#[test]
fn pack_nil() {
    let mut buf = [0x00];

    let val = Value::Nil;

    write_value(&mut &mut buf[..], &val).unwrap();

    assert_eq!([0xc0], buf);
}
