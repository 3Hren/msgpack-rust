# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Planning][planning]
- FixedValue* -> FixVal*
- ValueRef to Value conversion.
- Value to ValueRef casting.
- Check all TODO's.

## [Unreleased][unreleased]
### Added
- Implemented `std::error::Error` trait for error types.
- New `ValueRef` value struct represents MessagePack'ed value, but unlike an owning `Value` it owns nothing except its
  structure. It means that all strings/binaries it contains are borrowed from the byte array from which the value was
  created.
- Encoding function for `ValueRef`.
- Decoding function for `ValueRef`.
- Conversion method from `ValueRef` to `Value`.
- More benchmarks and tests.

### Changed
- Derive `Copy` trait for `Integer` and `Float` enums.

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
