//! ## Examples
//!
//! Let's try to encode a tuple of int and string.
//!
//! ```rust
//! extern crate rmp_serialize;
//! extern crate rustc_serialize;
//!
//! use rustc_serialize::Encodable;
//! use rmp_serialize::Encoder;
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
//! extern crate rmp_serialize;
//! extern crate rustc_serialize;
//!
//! use rustc_serialize::Decodable;
//! use rmp_serialize::Decoder;
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
//! extern crate rmp_serialize;
//! extern crate rustc_serialize;
//!
//! use rustc_serialize::{Encodable, Decodable};
//! use rmp_serialize::{Encoder, Decoder};
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

extern crate rmp;
extern crate rustc_serialize;

// pub mod decode;
pub mod encode;

// pub use decode::Decoder;
pub use encode::Encoder;
