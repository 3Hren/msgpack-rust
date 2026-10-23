use rmp::decode::{LenError, MessageLen};

// fixext1: marker, type, 1 data byte
#[test]
fn repro_len_of_fixext1() {
    assert_eq!(MessageLen::len_of(&[0xd4, 0x01, 0x2a, 0xff]).unwrap(), 3);
}

// [ext8 len=1 type=1 data]
#[test]
fn repro_len_of_ext8_in_array() {
    assert_eq!(MessageLen::len_of(&[0x91, 0xc7, 0x01, 0x01, 0x2a, 0xff]).unwrap(), 5);
}

#[test]
fn repro_len_of_ext16_truncated() {
    // ext16 len=4: needs 1 + 2 + 1 + 4 = 8 bytes
    match MessageLen::len_of(&[0xc8, 0x00, 0x04, 0x01, 0x00]) {
        Err(LenError::Truncated(n)) => assert!(n.get() <= 8),
        other => panic!("{other:?}"),
    }
}

#[test]
fn repro_len_of_matches_encoder() {
    let mut buf = Vec::new();
    rmp::encode::write_array_len(&mut buf, 3).unwrap();
    rmp::encode::write_ext_meta(&mut buf, 3, 5).unwrap();
    buf.extend_from_slice(&[1, 2, 3]);
    rmp::encode::write_ext_meta(&mut buf, 300, -1).unwrap();
    buf.extend_from_slice(&[0; 300]);
    rmp::encode::write_ext_meta(&mut buf, 16, 7).unwrap();
    buf.extend_from_slice(&[9; 16]);
    let total = buf.len();
    buf.push(0xc0);
    assert_eq!(MessageLen::len_of(&buf).unwrap(), total);
    let mut inc = MessageLen::new();
    let mut res = Err(LenError::ParseError);
    for b in buf.chunks(1) {
        res = inc.incremental_len(b);
        if res.is_ok() {
            break;
        }
    }
    assert_eq!(res.unwrap(), total);
}
