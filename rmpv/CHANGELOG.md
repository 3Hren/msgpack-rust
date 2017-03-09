# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## Unreleased

## 0.3.0 - 2017-03-09
### Added
- Implement `Deserializer` and `Serializer` for `Value`.
- Add `kind()` method for `rmpv::decode::Error`.
- Implement `Error` and `Display` traits for `rmpv::decode::Error`.
- Implement `From` trait for `Value` and `ValueRef` from all integral types, strings, slices and other more.

### Changed
- Reserved markers are now decoded as nil instead of raising `Error::TypeMismatch`.
- Integer representation for `Value` and `ValueRef` has been changed and hidden from the user to be able to fully match the spec and to fix round-trip cases.
- Invalid UTF-8 strings can now be properly decoded into `Value` and `ValueRef` to match the spec. An untouched bytes can also be returned on demand as like as `Utf8Error` with description where invalid byte-sequence happened.
- Error enums for decoding `Value` and `ValueRef` has been merged into the single one, which is located at `rmpv::decode::Error`.

### Removed
- Remove `TypeMismatch` variant from `value::decode::Error`, because there is no way to obtain it.
- Remove `FromUtf8Error` variant from `value::decode::Error`, because there invalid UTF-8 sequences are now supported.

## 0.2.0 - 2017-02-09
### Added
- `Serde` 0.9 support.
- `ValueRef` can now be displayed.
- `ValueRef` can be indexed using special `index(..)` method. Implementing `Index` trait is not possible due to conflicting signature - `ValueRef` requires explicit lifetime.
- It's now possible to obtain `ErrorKind` for Errors.

## 0.1.0 - 2017-01-05
### Removed
- Value now saves integer and floating point numbers directly without intermediate `Integer` and `Float` enums. As a result, they were removed.
