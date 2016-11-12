extern crate rmp_value;

use rmp_value::decode::read_value;

#[test]
fn from_null_decode_value() {
    let buf = [0xc0];

    assert_eq!(Value::Nil, read_value(&mut buf[..]).unwrap());
}
