use std::io::Cursor;

use serde::Deserialize;

use msgpack::Deserializer;

#[test]
fn pass_deserializer_get_ref() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);

    assert_eq!((), Deserialize::deserialize(&mut de).ok().unwrap());
    assert_eq!(1, de.get_ref().position());
}

#[test]
fn pass_deserializer_get_mut() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);

    assert_eq!((), Deserialize::deserialize(&mut de).ok().unwrap());
    de.get_mut().set_position(0);

    assert_eq!((), Deserialize::deserialize(&mut de).ok().unwrap());
}

#[test]
fn pass_deserializer_into_inner() {
    let buf = [0xc0];
    let cur = Cursor::new(&buf[..]);

    let mut de = Deserializer::new(cur);

    assert_eq!((), Deserialize::deserialize(&mut de).ok().unwrap());
    let cur = de.into_inner();

    assert_eq!(1, cur.position());
}
