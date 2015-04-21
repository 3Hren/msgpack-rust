#![crate_name = "msgpack"]

#![unstable = "this library is still in rapid development"]

#![feature(core)]
#![feature(collections)]
#![cfg_attr(test, feature(test))]

extern crate byteorder;
extern crate rustc_serialize as serialize;

pub const MSGPACK_VERSION : u32 = 5;

pub mod core;

pub use core::{
    Marker,
};

pub use core::encode;

pub use core::decode::serialize::{
    Decoder,
};

/// Temporary. Move to error.rs module.
pub use core::{Error, ReadError};

#[cfg(test)]
mod bench;

// Stage 1. Low-level decoding functions.
// Stage 2. Value decoding functions.
// Stage 3. Deserialization.
// Stage 4. Low-level encoding functions.
// Stage 5. Value encoding functions.
// Stage 6. Serialization.
