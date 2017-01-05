#[cfg(feature = "with-serde")]
extern crate serde;
extern crate rmp_serde;
extern crate rmpv;

#[cfg(feature = "with-serde")]
mod tests {

use serde::Serialize;

use rmp_serde::Serializer;
use rmpv::Value;

#[test]
fn pass_value_nil() {
    let mut buf = Vec::new();

    Value::Nil.serialize(&mut Serializer::new(&mut buf)).unwrap();

    assert_eq!(vec![0xc0], buf);
}

#[test]
fn pass_value_bool() {
    let mut buf = Vec::new();
    {
        let mut encoder = Serializer::new(&mut buf);

        let val = Value::from(true);
        val.serialize(&mut encoder).unwrap();

        let val = Value::from(false);
        val.serialize(&mut encoder).unwrap();
    }

    assert_eq!(vec![0xc3, 0xc2], buf);
}

#[test]
fn pass_value_usize() {
    check_ser(Value::from(255usize), &[0xcc, 0xff]);
}

#[test]
fn pass_value_isize() {
    check_ser(Value::from(-128isize), &[0xd0, 0x80]);
}

#[test]
fn pass_value_f32() {
    check_ser(Value::from(3.4028234e38_f32), &[0xca, 0x7f, 0x7f, 0xff, 0xff]);
}

#[test]
fn pass_value_f64() {
    check_ser(Value::from(42.0), &[0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
}

#[test]
fn pass_value_string() {
    check_ser(Value::String("le message".into()),
        &[0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65]);
}

#[test]
fn pass_value_bin() {
    check_ser(Value::Binary(vec![0xcc, 0x80]), &[0xc4, 0x02, 0xcc, 0x80]);
}

#[test]
fn pass_value_array() {
    check_ser(Value::Array(vec![Value::String("le".into()), Value::String("shit".into())]),
        &[0x92, 0xa2, 0x6c, 0x65, 0xa4, 0x73, 0x68, 0x69, 0x74]);
}

#[test]
fn pass_value_map() {
    let val = Value::Map(vec![
        (Value::from(0), Value::String("le".into())),
        (Value::from(1), Value::String("shit".into())),
    ]);

    let out = [
        0x82, // 2 (size)
        0x00, // 0
        0xa2, 0x6c, 0x65, // "le"
        0x01, // 1
        0xa4, 0x73, 0x68, 0x69, 0x74, // "shit"
    ];

    check_ser(val, &out);
}

fn check_ser<T>(val: T, expected: &[u8])
    where T: Serialize
{
    let mut buf = Vec::new();
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();
    assert_eq!(expected, &buf[..]);
}

}
