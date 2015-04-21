use msgpack::core::encode::*;

#[test]
fn pass_pack_pfix() {
    let mut buf = [0x00];

    write_pfix(&mut &mut buf[..], 127).ok().unwrap();

    assert_eq!([0x7f], buf);
}
