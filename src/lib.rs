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
//! rmp = "0.5.0"
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
//! Now we have an encoded buffer, which we can decode the same way:
//!
//! ```rust
//! extern crate rmp as msgpack;
//! extern crate rustc_serialize;
//!
//! use rustc_serialize::Decodable;
//! use msgpack::Decoder;
//!
//! fn main() {
//!     let buf = [0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72];
//!
//!     let mut decoder = Decoder::new(&buf[..]);
//!
//!     let res: (u8, String) = Decodable::decode(&mut decoder).unwrap();
//!
//!     assert_eq!((42u8, "the Answer".to_string()), res);
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
//! use rustc_serialize::{Encodable, Decodable};
//! use msgpack::{Encoder, Decoder};
//!
//! #[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
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
//!
//!     // Now try to unpack the buffer into the initial struct.
//!     let mut decoder = Decoder::new(&buf[..]);
//!     let res: Custom = Decodable::decode(&mut decoder).ok().unwrap();
//!
//!     assert_eq!(val, res);
//! }
//! ```

extern crate byteorder;
extern crate rustc_serialize as serialize;

extern crate serde;

pub mod encode;
pub mod decode;

mod init;

pub mod value;

pub const MSGPACK_VERSION : u32 = 5;

pub use decode::serialize::Decoder;
pub use encode::serialize::Encoder;

pub use decode::serde::Deserializer;
pub use encode::serde::Serializer;

pub use init::Marker;

pub use value::{Value, ValueRef};
