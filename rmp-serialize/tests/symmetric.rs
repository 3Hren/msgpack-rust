extern crate rmp_serialize;
extern crate rustc_serialize;

use rustc_serialize::Decodable;
use rustc_serialize::Encodable;

use rmp_serialize::Decoder;
use rmp_serialize::Encoder;

#[test]
fn pass_symmetric_0i32() {
    let val = 0i32;
    let mut buf = Vec::new();

    val.encode(&mut Encoder::new(&mut buf)).unwrap();
    let mut decoder = Decoder::new(&buf[..]);

    let res: i32 = Decodable::decode(&mut decoder).unwrap();

    assert_eq!(val, res);
}

#[test]
fn pass_generalized_integer_decoding() {
    // Note, that we encode the value as `i64` and decode it as `u16`.
    let val = 10050i64;
    let mut buf = Vec::new();

    val.encode(&mut Encoder::new(&mut buf)).unwrap();
    let mut decoder = Decoder::new(&buf[..]);

    let res: u16 = Decodable::decode(&mut decoder).unwrap();

    assert_eq!(val, res as i64);
}
