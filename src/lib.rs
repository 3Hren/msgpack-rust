#![crate_name = "msgpack"]

#![unstable = "this library is still in rapid development"]

#![feature(core)]
#![feature(collections)]
#![cfg_attr(test, feature(test))]

extern crate byteorder;
extern crate rustc_serialize as serialize;

pub const MSGPACK_VERSION : u32 = 5;

pub mod core;

pub use core::decode::{
    read_nil,
};

pub use core::decode::serialize::{
    Decoder,
};

/// Temporary. Move to error.rs module.
pub use core::{Error, ReadError, MarkerError};

#[cfg(test)]
mod bench;
