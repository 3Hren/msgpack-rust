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
//! rmp = "0.7.2"
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

extern crate byteorder;

pub mod encode;
pub mod decode;

mod init;

pub mod value;

pub const MSGPACK_VERSION : u32 = 5;

pub use init::Marker;

pub use value::{Value, ValueRef};
