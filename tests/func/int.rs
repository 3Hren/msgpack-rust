use msgpack::core::encode::*;

#[test]
fn pass_pack_pfix() {
    let mut buf = [0x00];

    assert_eq!(1, write_pfix(&mut &mut buf[..], 127).unwrap());
    assert_eq!([0x7f], buf);
}
