use msgpack::core::encode::*;

#[test]
fn pass_pack_pfix() {
    let mut buf = [0x00];

    write_pfix(&mut &mut buf[..], 127).ok().unwrap();

    assert_eq!([0x7f], buf);
}

#[test]
fn fail_pack_pfix_too_small_buffer() {
    let mut buf = [];

    match write_pfix(&mut &mut buf[..], 127) {
        Err(Error::InvalidFixedValueWrite(..)) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn fail_pack_pfix_too_large() {
    let mut buf = [0x00];

    match write_pfix(&mut &mut buf[..], 128) {
        Err(Error::TypeMismatch) => (),
        other => panic!("unexpected result: {:?}", other)
    }
}

#[test]
fn pass_pack_u8() {
    let mut buf = [0x00, 0x00];

    write_u8(&mut &mut buf[..], 127).ok().unwrap();

    assert_eq!([0xcc, 0x7f], buf);
}
