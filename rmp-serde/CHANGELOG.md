# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased][unreleased]
### Changed
- Serializer can now be extended with custom struct encoding policy.

### Fixed
- Deserializer now properly works with enums.
- Options with default values (that can be initialized using unit marker) deserialization.
  This fix also forbids the following Option deserialization cases:
    - Option<()>.
    - Option<Option<...>>.
  It's impossible to properly deserialize the listed cases without explicit option marker in protocol.
