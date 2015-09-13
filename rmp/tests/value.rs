extern crate rmp;

use rmp::Value;
use rmp::value::{Float, Integer};

#[test]
fn display_nil() {
    assert_eq!("nil", format!("{}", Value::Nil));
}

#[test]
fn display_bool() {
    assert_eq!("true", format!("{}", Value::Boolean(true)));
    assert_eq!("false", format!("{}", Value::Boolean(false)));
}

#[test]
fn display_int() {
    assert_eq!("42", format!("{}", Value::Integer(Integer::U64(42))));
    assert_eq!("42", format!("{}", Value::Integer(Integer::I64(42))));
}

#[test]
fn display_float() {
    assert_eq!("3.1415", format!("{}", Value::Float(Float::F32(3.1415))));
    assert_eq!("3.1415", format!("{}", Value::Float(Float::F64(3.1415))));
}

#[test]
fn display_string() {
    assert_eq!("\"le string\"", format!("{}", Value::String("le string".to_owned())));
}

#[test]
fn display_binary() {
    assert_eq!("[108, 101, 32, 115, 116, 114, 105, 110, 103]", format!("{}",
        Value::Binary(b"le string".to_vec())));
}

#[test]
fn display_array() {
    assert_eq!("[]", format!("{}", Value::Array(vec![])));
    assert_eq!("[nil]", format!("{}", Value::Array(vec![Value::Nil])));
    assert_eq!("[nil, nil]", format!("{}", Value::Array(vec![Value::Nil, Value::Nil])));
}

#[test]
fn display_map() {
    assert_eq!("{}", format!("{}", Value::Map(vec![])));
    assert_eq!("{nil: nil}", format!("{}", Value::Map(vec![(Value::Nil, Value::Nil)])));
    assert_eq!("{nil: nil, true: false}", format!("{}", Value::Map(vec![(Value::Nil, Value::Nil),
        (Value::Boolean(true), Value::Boolean(false))])));
}
