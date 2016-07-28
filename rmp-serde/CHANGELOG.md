# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased][unreleased]
## 0.9.5 - 2016-07-28
### Added
- Added a wrapper over rmp::Value to be able to serialize it.

## 0.9.4 - 2016-07-11
### Fixed
- Reading binary should no longer trigger unexpected EOF error on valid read.

## 0.9.3 - 2016-07-11
### Changed
- Reuse deserializer buffer on every read for string and binary deserialization without unnecessary intermediate buffer creation.
  This change increases the string and binary deserialization performance (many thanks to Fedor Gogolev <knsd@knsd.net>).

## 0.9.2 - 2016-07-03
### Added
- Implement `size_hint()` function for `SeqVisitor` and `MapVisitor`, so it can be possible to preallocate things, increasing the performance greatly.

## 0.9.1 - 2016-06-24
### Fixed
- Serializer should no longer panic with unimplemented error on struct variant serialization ([#64]).

## 0.9.0 - 2016-03-28
### Changed
- Adapt code to be compilable with Serde v0.7.

## 0.8.2 - 2015-11-10
### Changed
- Fixed stack overflow when unpacking recursive data structures.

## 0.8.1 - 2015-10-03
### Changed
- Upper limit for serde version.

### Fixed
- Use the most effective int encoding
  Even if the value is explicitly marked as i64 it must be encoded using
  the most effective bytes representation despite of signed it or
  unsigned.

## 0.8.0 - 2015-09-11
### Changed
- Serializer can now be extended with custom struct encoding policy.
- Improved error types and its messages for serialization part.
    - New error type introduced - UnknownLength. Returned on attempt to serialize struct, map or serquence with unknown
    length (Serde allows this).
    - The new type is returned if necessary.

### Fixed
- Deserializer now properly works with enums.
- Options with default values (that can be initialized using unit marker) deserialization.
  This fix also forbids the following Option deserialization cases:
    - Option<()>.
    - Option<Option<...>>.
  It's impossible to properly deserialize the listed cases without explicit option marker in protocol.
- Serializer now properly serializes unit structs.
  Previously it was serialized as a unit (nil), now there is just an empty array ([]).

[#64]: (https://github.com/3Hren/msgpack-rust/pull/64)
