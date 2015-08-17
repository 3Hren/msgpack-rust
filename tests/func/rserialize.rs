use std::io::Cursor;

use rustc_serialize::Decodable;

use msgpack::Decoder;

#[test]
fn pass_decoder_get_ref() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
    assert_eq!(1, decoder.get_ref().position());
}

#[test]
fn pass_decoder_get_mut() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
    decoder.get_mut().set_position(0);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
}

#[test]
fn pass_decoder_into_inner() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut decoder = Decoder::new(cur);

    assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
    let cur = decoder.into_inner();

    assert_eq!(1, cur.position());
}
