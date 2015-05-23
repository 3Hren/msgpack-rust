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
//! rmp = "0.1.1"
//! ```
//!
//! Then, add this to your crate root:
//!
//! ```rust
//! extern crate rmp as msgpack;
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
//! - **Clear error handling**
//!
//!   RMP's error system guarantees that you never receive an error enum with unreachable variant.
//!
//! - **Robust and tested**
//!
//!   This project is developed using TDD and CI, so any found bugs will be fixed without breaking
//!   existing functionality.
//!
//! ## Examples
//!
//! Let's try to encode a tuple of int and string.
//!
//! ```rust
//! extern crate rmp as msgpack;
//! extern crate rustc_serialize;
//!
//! use rustc_serialize::Encodable;
//! use msgpack::Encoder;
//!
//! fn main() {
//!     let val = (42u8, "the Answer");
//!
//!     // The encoder borrows the bytearray buffer.
//!     let mut buf = [0u8; 13];
//!
//!     val.encode(&mut Encoder::new(&mut &mut buf[..]));
//!
//!     assert_eq!([0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72], buf);
//! }
//! ```
//!
//! RMP also allows to automatically serialize/deserialize custom structures using rustc_serialize
//! reflection. To enable this feature, derive RustcEncodable and RustcDecodable attributes as
//! shown in the following example:
//!
//! ```rust
//! extern crate rmp as msgpack;
//! extern crate rustc_serialize;
//!
//! use rustc_serialize::Encodable;
//! use msgpack::Encoder;
//!
//! #[derive(RustcEncodable)]
//! struct Custom {
//!     id: u32,
//!     key: String,
//! }
//!
//! fn main() {
//!     let val = Custom { id: 42u32, key: "the Answer".to_string() };
//!
//!     let mut buf = [0u8; 13];
//!
//!     val.encode(&mut Encoder::new(&mut &mut buf[..]));
//!
//!     assert_eq!([0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72], buf);
//! }
//! ```
//!
//! ## Limitations and plans
//!
//! - Owning `Value` variant and its encoding function.
//! - Non-owning `ValueRef` variant, which can be created from `[u8]`, `Cursor<[u8]>` etc. and
//!   borrows data from it, which makes it absolute zero-copy.
//! - Enum serialization/deserialization.

extern crate byteorder;
extern crate rustc_serialize as serialize;

pub const MSGPACK_VERSION : u32 = 5;

pub use init::Marker;

pub use decode::serialize::{
    Decoder,
};

pub use encode::serialize::{
    Encoder,
};

mod init;
pub mod encode;
pub mod decode;

// TODO: Not ready yet.
mod value;

// Suppressed due to instability.
// #[cfg(test)]
// mod bench;
