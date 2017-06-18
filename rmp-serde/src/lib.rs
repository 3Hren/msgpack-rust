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
//! extern crate rmp_serde as rmps;
//!
//! use serde::{Deserialize, Serialize};
//! use rmps::{Deserializer, Serializer};
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
//! extern crate serde;
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate rmp_serde as rmps;
//!
//! use std::collections::HashMap;
//! use serde::{Deserialize, Serialize};
//! use rmps::{Deserializer, Serializer};
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

use std::fmt::{self, Display, Formatter};
use std::str::{self, Utf8Error};

use serde::de::{self, Deserialize};

pub use decode::Deserializer;
pub use encode::Serializer;

pub mod decode;
pub mod encode;

/// Helper that allows to decode strings no matter whether they contain valid or invalid UTF-8.
#[derive(Clone, Debug, PartialEq)]
pub struct Raw {
    s: Result<String, (Vec<u8>, Utf8Error)>,
}

impl Raw {
    /// Returns `true` if the raw is valid UTF-8.
    pub fn is_str(&self) -> bool {
        self.s.is_ok()
    }

    /// Returns `true` if the raw contains invalid UTF-8 sequence.
    pub fn is_err(&self) -> bool {
        self.s.is_err()
    }

    /// Returns the string reference if the raw is valid UTF-8, or else `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self.s {
            Ok(ref s) => Some(s.as_str()),
            Err(..) => None,
        }
    }

    /// Returns the underlying `Utf8Error` if the raw contains invalid UTF-8 sequence, or
    /// else `None`.
    pub fn as_err(&self) -> Option<&Utf8Error> {
        match self.s {
            Ok(..) => None,
            Err((_, ref err)) => Some(&err),
        }
    }

    /// Returns a byte slice of this raw's contents.
    pub fn as_bytes(&self) -> &[u8] {
        match self.s {
            Ok(ref s) => s.as_bytes(),
            Err(ref err) => &err.0[..],
        }
    }

    /// Consumes this object, yielding the string if the raw is valid UTF-8, or else `None`.
    pub fn into_str(self) -> Option<String> {
        self.s.ok()
    }

    /// Converts a `Raw` into a byte vector.
    pub fn into_bytes(self) -> Vec<u8> {
        match self.s {
            Ok(s) => s.into_bytes(),
            Err(err) => err.0,
        }
    }
}

struct RawVisitor;

impl<'de> de::Visitor<'de> for RawVisitor {
    type Value = Raw;

    fn expecting(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        "string or bytes".fmt(fmt)
    }

    #[inline]
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
        Ok(Raw { s: Ok(v) })
    }

    #[inline]
    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(Raw { s: Ok(v.into()) })
    }

    #[inline]
    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where E: de::Error
    {
        let s = match str::from_utf8(v) {
            Ok(s) => Ok(s.into()),
            Err(err) => Err((v.into(), err)),
        };

        Ok(Raw { s: s })
    }

    #[inline]
    fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E>
        where E: de::Error
    {
        let s = match String::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(err) => {
                let e = err.utf8_error();
                Err((err.into_bytes(), e))
            }
        };

        Ok(Raw { s: s })
    }
}

impl<'de> Deserialize<'de> for Raw {
    #[inline]
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        de.deserialize_any(RawVisitor)
    }
}

/// Helper that allows to decode strings no matter whether they contain valid or invalid UTF-8.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RawRef<'a> {
    s: Result<&'a str, (&'a [u8], Utf8Error)>,
}

impl<'a> RawRef<'a> {
    /// Returns `true` if the raw is valid UTF-8.
    pub fn is_str(&self) -> bool {
        self.s.is_ok()
    }

    /// Returns `true` if the raw contains invalid UTF-8 sequence.
    pub fn is_err(&self) -> bool {
        self.s.is_err()
    }

    /// Returns the string reference if the raw is valid UTF-8, or else `None`.
    pub fn as_str(&self) -> Option<&str> {
        match self.s {
            Ok(ref s) => Some(s),
            Err(..) => None,
        }
    }

    /// Returns the underlying `Utf8Error` if the raw contains invalid UTF-8 sequence, or
    /// else `None`.
    pub fn as_err(&self) -> Option<&Utf8Error> {
        match self.s {
            Ok(..) => None,
            Err((_, ref err)) => Some(&err),
        }
    }

    /// Returns a byte slice of this raw's contents.
    pub fn as_bytes(&self) -> &[u8] {
        match self.s {
            Ok(ref s) => s.as_bytes(),
            Err(ref err) => &err.0[..],
        }
    }
}

struct RawRefVisitor;

impl<'de> de::Visitor<'de> for RawRefVisitor {
    type Value = RawRef<'de>;

    fn expecting(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        "string or bytes".fmt(fmt)
    }

    #[inline]
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where E: de::Error
    {
        Ok(RawRef { s: Ok(v) })
    }

    #[inline]
    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
        where E: de::Error
    {
        let s = match str::from_utf8(v) {
            Ok(s) => Ok(s),
            Err(err) => Err((v, err)),
        };

        Ok(RawRef { s: s })
    }
}

impl<'de> Deserialize<'de> for RawRef<'de> {
    #[inline]
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
        where D: de::Deserializer<'de>
    {
        de.deserialize_any(RawRefVisitor)
    }
}

// Reexport common functions from encode module
pub use encode::{write, write_named, to_vec, to_vec_named};
// Reexport common functions from decode module
pub use decode::{from_slice, from_read};
