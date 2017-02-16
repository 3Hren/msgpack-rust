# Change Log
All notable changes to this project will be documented in this file.
This project adheres to [Semantic Versioning](http://semver.org/).

## Unreleased
### Added
- `kind()` method for `Value` decode `Error`.

## 0.2.0 - 2017-02-09
### Added
- `Serde` 0.9 support.
- `ValueRef` can now be displayed.
- `ValueRef` can be indexed using special `index(..)` method. Implementing `Index` trait is not possible due to conflicting signature - `ValueRef` requires explicit lifetime.
- It's now possible to obtain `ErrorKind` for Errors.

## 0.1.0 - 2017-01-05
### Removed
- Value now saves integer and floating point numbers directly without intermediate `Integer` and `Float` enums. As a result, they were removed.
