# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased][unreleased]

## 0.7.2 - 2015-09-23
### Added
- Implemented `Display` trait for `Value`.

## 0.7.1 - 2015-09-11
### Changed
- Use `to_owned` instead of `to_string` while converting `ValueRef` into `Value`.
  This change improves `ValueRef::to_owned()` method performance by approximately 10-20%.
  Also after this commit it's cheaper to decode directly into `ValueRef` with further converting to owned value rather
  than decoding directly into `Value`.

## 0.7.0 - 2015-08-24
### Changed
- The big single crate has been refactored, which results in three crates: `rmp`, `rmp-serialize` and `rmp-serde`.

## 0.6.0 - 2015-08-17
### Added
- Initial support for [Serde](https://github.com/serde-rs/serde) serializer and deserializer.
- Efficient bytes serialization with Serde.
- Efficient binaries deserialization with Serde using `ByteBuf`.
- Rust serialize Decoder now can provide the underlying reader both by reference or by value, destroying itself in the
  last case.

### Changed
- Update well-formness for `BigEndianRead` trait to be implemented only for sized types.
- Renamed `PositiveFixnum` marker to `FixPos`.
- Renamed `NegativeFixnum` marker to `FixNeg`.
- Renamed `FixedString` marker to `FixStr`.
- Renamed `FixedArray` marker to `FixArray`.
- Renamed `FixedMap` to `FixMap`.
- Minor documentation updates and markdown fixes.

## 0.5.1 - 2015-08-10
### Changed
- Now the `rustc_serialize::Encoder` should encode signed integers using the most effective underlying representation.
- Now the `rustc_serialize::Decoder` should properly map integers to the result type if the decoded value fits in
  result type's range.

## 0.5.0 - 2015-08-01
### Added
- New `ValueRef` value struct represents MessagePack'ed value, but unlike an owning `Value` it owns nothing except its
  structure. It means that all strings and binaries it contains are borrowed from the byte array from which the value
  was created.
- New `BorrowRead` trait, which looks like a standard `BufRead` but unlike the standard this has an explicit internal
  buffer lifetime, which allows to borrow from underlying buffer while mutating the type.
- Encoding function for `ValueRef` with its own error category.
- Decoding function for `ValueRef` with its own error category.
- Conversion method from `ValueRef` to `Value`.
- More benchmarks and tests.

### Changed
- Derive `Copy` trait for `Integer` and `Float` enums.

## 0.4.0 - 2015-07-17
### Added
- Low level `write_str` function allows to serialize the UTF-8 encoded strings the most efficient way.
- Low level `write_bin` function allows to serialize the binary array the most efficient way.
- Implemented `std::error::Error` trait for error types.

## 0.3.2 - 2015-07-05
### Changed
- Encoder now should return proper error types.

## 0.3.1 - 2015-06-28
### Changed
- Stabilizing enum serialization/deserialization. Now every enum is serialized as [int, [args...]].
- Suppressed some warnings appeared on updated compiler.

## 0.3.0 - 2015-06-25
### Added
- Enum serialization/deserialization.

## 0.2.2 - 2015-06-15
### Changed
- Minor integer decoding performance tweaking.

## 0.2.1 - 2015-05-30
### Added
 - Benchmarking module.

### Changed
- Increased string decoding performance by ~30 times.
- Exported `read_value` function to the `rmp::decode` module.
- Exported `Value` struct to the root crate namespace.

## 0.2.0 - 2015-05-27
### Added
- Introducing a `Value` algebraic data type, which represents an owning MessagePack object. It can
  be found in `rmp::value` module.
- The Value ADT encoding and decoding functions.
- Low-level ext type decoders.

## 0.1.1 - 2015-05-18
### Changed
- Added documentation and repository site in Cargo.toml.
- Added keywords to ease searching using crates.io.

## 0.1.0 - 2015-05-15
### Added
- Initial commit.
- This CHANGELOG file to hopefully serve as an evolving example of a standardized open source
  project CHANGELOG.
