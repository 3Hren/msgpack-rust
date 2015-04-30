use rustc_serialize::Encodable;

use msgpack::Encoder;

#[test]
fn encode_null() {
    let mut buf = [0x00];

    let val = ();
    val.encode(&mut Encoder::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xc0], buf);
}
