#[test]
fn pass_value_nil() {
    let mut buf = [0x00];

    let val = Value(rmp::Value::Nil);
    val.serialize(&mut Serializer::new(&mut &mut buf[..])).ok().unwrap();

    assert_eq!([0xc0], buf);
}

#[test]
fn pass_value_bool() {
    let mut buf = [0x00, 0x00];

    {
        let mut cur = Cursor::new(&mut buf[..]);
        let mut encoder = Serializer::new(&mut cur);

        let val = Value::from(true);
        val.serialize(&mut encoder).ok().unwrap();

        let val = Value::from(false);
        val.serialize(&mut encoder).ok().unwrap();
    }

    assert_eq!([0xc3, 0xc2], buf);
}

#[test]
fn pass_value_usize() {
    check_ser(Value::from(255usize), &mut [0x00, 0x00], &[0xcc, 0xff]);
}

#[test]
fn pass_value_isize() {
    check_ser(Value::from(-128isize), &mut [0x00, 0x00], &[0xd0, 0x80]);
}

#[test]
fn pass_value_f32() {
    check_ser(Value::from(3.4028234e38_f32), &mut [0x00, 0x00, 0x00, 0x00, 0x00],
        &[0xca, 0x7f, 0x7f, 0xff, 0xff]);
}

#[test]
fn pass_value_f64() {
    check_ser(Value::from(42.0), &mut [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn pass_value_string() {
    check_ser(Value::from(rmp::Value::String("le message".into())),
    &mut [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65]);
}

#[test]
fn pass_value_bin() {
    check_ser(Value::from(rmp::Value::Binary(vec![0xcc, 0x80])),
    &mut [0x00, 0x00, 0x00, 0x00],
        &[0xc4, 0x02, 0xcc, 0x80]);
}

#[test]
fn pass_value_array() {
    check_ser(Value::from(rmp::Value::Array(vec![rmp::Value::String("le".into()), rmp::Value::String("shit".into())])),
    &mut [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        &[0x92, 0xa2, 0x6c, 0x65, 0xa4, 0x73, 0x68, 0x69, 0x74]);
}

#[test]
fn pass_value_map() {
    let val = rmp::Value::Map(vec![
        (rmp::Value::from(0), rmp::Value::String("le".into())),
        (rmp::Value::from(1), rmp::Value::String("shit".into())),
    ]);

    let out = [
        0x82, // 2 (size)
        0x00, // 0
        0xa2, 0x6c, 0x65, // "le"
        0x01, // 1
        0xa4, 0x73, 0x68, 0x69, 0x74, // "shit"
    ];

    check_ser(Value(val),
        &mut [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        &out);
}

fn check_ser<T>(val: T, buf: &mut [u8], expected: &[u8])
    where T: Serialize
{
    {
        let mut cur = Cursor::new(&mut buf[..]);
        let mut encoder = Serializer::new(&mut cur);

        val.serialize(&mut encoder).unwrap();
    };

    assert_eq!(expected, buf);
}
