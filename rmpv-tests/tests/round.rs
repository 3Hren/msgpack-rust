#[macro_use]
extern crate serde_derive;
extern crate rmp_serde as rmps;

use std::fmt::Debug;

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_bytes::ByteBuf;

use rmpv::Value;

/// Tests that the following round-trip conditions are met:
/// - `T`     -> `[u8]`  == `Value` -> `[u8]`.
/// - `T`     -> `Value` == `Value`.
/// - `[u8]`  -> `T`     == `T`.
/// - `[u8]`  -> `Value` == `Value`.
/// - `Value` -> `T`     == `T`.
fn test_round<'de, T>(var: T, val: Value)
    where T: Debug + PartialEq + Serialize + DeserializeOwned
{
    // Serialize part.
    // Test that `T` -> `[u8]` equals with serialization from `Value` -> `[u8]`.
    let buf_from_var = rmps::to_vec(&var).unwrap();
    let buf_from_val = rmps::to_vec(&val).unwrap();
    assert_eq!(buf_from_var, buf_from_val);

    // Test that `T` -> `Value` equals with the given `Value`.
    let val_from_var = rmpv::ext::to_value(&var).unwrap();
    assert_eq!(val, val_from_var);

    // Deserialize part.
    // Test that `[u8]` -> `T` equals with the given `T`.
    let var_from_buf: T = rmps::from_slice(buf_from_var.as_slice()).unwrap();
    assert_eq!(var, var_from_buf);

    // Test that `[u8]` -> `Value` equals with the given `Value`.
    let val_from_buf: Value = rmps::from_slice(buf_from_var.as_slice()).unwrap();
    assert_eq!(val, val_from_buf);

    // Test that `Value` -> `T` equals with the given `T`.
    let var_from_val: T = rmpv::ext::from_value(val_from_buf).unwrap();
    assert_eq!(var, var_from_val);
}

#[test]
fn pass_nil() {
    test_round((), Value::Nil);
}

#[test]
fn pass_bool() {
    test_round(true, Value::Boolean(true));
    test_round(false, Value::Boolean(false));
}

#[test]
fn pass_uint() {
    test_round(u8::min_value(), Value::from(u8::min_value()));
    test_round(u8::max_value(), Value::from(u8::max_value()));
    test_round(u16::max_value(), Value::from(u16::max_value()));
    test_round(u32::max_value(), Value::from(u32::max_value()));
    test_round(u64::max_value(), Value::from(u64::max_value()));
}

#[test]
fn pass_sint() {
    test_round(i8::min_value(), Value::from(i8::min_value()));
    test_round(i8::max_value(), Value::from(i8::max_value()));
    test_round(i16::min_value(), Value::from(i16::min_value()));
    test_round(i16::max_value(), Value::from(i16::max_value()));
    test_round(i32::min_value(), Value::from(i32::min_value()));
    test_round(i32::max_value(), Value::from(i32::max_value()));
    test_round(i64::min_value(), Value::from(i64::min_value()));
    test_round(i64::max_value(), Value::from(i64::max_value()));
}

#[test]
fn pass_f32() {
    test_round(std::f32::MAX, Value::from(std::f32::MAX));
}

#[test]
fn pass_f64() {
    test_round(0.00, Value::from(0.00));
    test_round(42.0, Value::from(42.0));
}

#[test]
fn pass_str() {
    test_round("John".to_string(), Value::from("John"));
}

#[test]
fn pass_bin() {
    test_round(ByteBuf::from(&[0xcc, 0x80][..]), Value::from(&[0xcc, 0x80][..]));
}

#[test]
fn pass_vec() {
    test_round([0, 42], Value::from(vec![Value::from(0), Value::from(42)]));
    test_round(vec![0, 42], Value::from(vec![Value::from(0), Value::from(42)]));
}

#[test]
fn pass_newtype_struct() {
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Newtype(String);

    test_round(Newtype("John".into()), Value::from("John"));
}
