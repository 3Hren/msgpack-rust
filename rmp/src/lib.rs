//! # The Rust MessagePack Library
//!
//! RMP is a pure Rust [MessagePack](http://msgpack.org) implementation.
//!
//! MessagePack is an efficient binary serialization format.
//!
//! **Warning** this library is still in rapid development and everything may change until 1.0 comes.
//!
//! ## Usage
//!
//! To use `rmp`, first add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies.rmp]
//! rmp = "^0.7"
//! ```
//!
//! Then, add this to your crate root:
//!
//! ```rust
//! extern crate rmp as msgpack; // Or just `rmp`.
//! ```
//!
//! ## Features
//!
//! - **Convenient API**
//!
//!   RMP is designed to be lightweight and straightforward. There are low-level API, which gives you
//!   full control on data encoding/decoding process and makes no heap allocations. On the other hand
//!   there are high-level API, which provides you convenient interface using Rust standard library and
//!   compiler reflection, allowing to encode/decode structures using `derive` attribute.
//!
//! - **Zero-copy value decoding**
//!
//!   RMP allows to decode bytes from a buffer in a zero-copy manner easily and blazingly fast, while Rust
//!   static checks guarantees that the data will be valid until buffer lives.
//!
//! - **Clear error handling**
//!
//!   RMP's error system guarantees that you never receive an error enum with unreachable variant.
//!
//! - **Robust and tested**
//!
//!   This project is developed using TDD and CI, so any found bugs will be fixed without breaking
//!   existing functionality.
//!
//! ## Detailed
//!
//! This crate represents the very basic functionality needed to work with MessagePack format.
//! Ideologically it is developed as a basis for building high-level abstractions.
//!
//! Currently there are three large modules: encode, decode and value. More detail you can find
//! in the corresponding sections.
//!
//! Formally every MessagePack message consists of some marker encapsulating a date type and the
//! data itself. Sometimes there are no separate data chunk, for example for booleans. In these
//! cases a marker contains the value. For example, the `true` value is encoded as `0xc3`.
//!
//! Also note, that a single value can be encoded in multiple ways. For example a value of `42` can
//! be represented as: `[0x2a], [0xcc, 0x2a], [0xcd, 0x00, 0x2a]` and so on.
//!
//! In these cases RMP guarantees that for encoding the most compact representation will be chosen.
//! On the other hand for deserialization it is not matter in which representation the value is
//! encoded - RMP deals with all of them.
//!
//! ## API
//!
//! Almost all API are represented as pure functions, which accepts a generic `Write` and the value.

extern crate byteorder;
extern crate num_traits;

mod marker;
pub mod encode;
pub mod decode;

pub use marker::Marker;

pub const MSGPACK_VERSION : u32 = 5;
