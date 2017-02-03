//! # Type-based Serialization and Deserialization
//!
//! Serde provides a mechanism for low boilerplate serialization & deserialization of values to and
//! from MessagePack via the serialization API. To be able to serialize a piece of data, it must
//! implement the `serde::Serialize` trait. To be able to deserialize a piece of data, it must
//! implement the `serde::Deserialize` trait. Serde provides provides an annotation to
//! automatically generate the code for these traits: `#[derive(Serialize, Deserialize)]`.
//!
//! # Examples
//!
//! Let's try to encode and decode some built-in types.
//!
//! ```rust
//! extern crate serde;
//! extern crate rmp_serde;
//!
//! use serde::{Deserialize, Serialize};
//! use rmp_serde::{Deserializer, Serializer};
//!
//! fn main() {
//!     let mut buf = Vec::new();
//!     let val = (42u8, "the Answer");
//!     val.serialize(&mut Serializer::new(&mut buf)).unwrap();
//!
//!     assert_eq!(vec![0x92, 0x2a, 0xaa, 0x74, 0x68, 0x65, 0x20, 0x41, 0x6e, 0x73, 0x77, 0x65, 0x72], buf);
//!
//!     let mut de = Deserializer::new(&buf[..]);
//!     assert_eq!((42, "the Answer".to_owned()), Deserialize::deserialize(&mut de).unwrap());
//! }
//! ```
//!
//! No one gonna hurt if we add some reflection magic.
//!
//! ```ignore
//! #![feature(proc_macro)]
//!
//! #[macro_use] extern crate serde_derive;
//! extern crate rmp_serde;
//!
//! use std::collections::HashMap;
//! use serde::{Deserialize, Serialize};
//! use rmp_serde::{Deserializer, Serializer};
//!
//! #[derive(Debug, PartialEq, Deserialize, Serialize)]
//! struct Human {
//!     age: u32,
//!     name: String,
//! }
//!
//! fn main() {
//!     let mut buf = Vec::new();
//!     let val = Human {
//!         age: 42,
//!         name: "John".into(),
//!     };
//!
//!     val.serialize(&mut Serializer::new(&mut buf)).unwrap();
//! }
//! ```

extern crate rmp;
extern crate byteorder;
#[macro_use]
extern crate serde;

pub use decode::Deserializer;
pub use encode::Serializer;

pub mod decode;
pub mod encode;

/// Serializes a value to a byte vector.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>, encode::Error>
    where T: serde::Serialize
{
    let mut buf = Vec::with_capacity(64);
    value.serialize(&mut Serializer::new(&mut buf))?;
    Ok(buf)
}

/// Deserializes a byte slice into the desired type.
pub fn from_slice<T>(input: &[u8]) -> Result<T, decode::Error>
    where T: serde::Deserialize
{
    let mut de = Deserializer::from_slice(input);
    serde::Deserialize::deserialize(&mut de)
}
