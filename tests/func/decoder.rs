use std::io::Cursor;

use serialize::Decodable;

use msgpack::Decoder;

#[test]
fn from_null_decode() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
}
