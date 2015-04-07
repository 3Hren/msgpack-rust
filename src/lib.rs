#![crate_name = "msgpack"]

#![unstable = "this library is still in rapid development"]

#![feature(core)]
#![feature(collections)]
#![cfg_attr(test, feature(test))]

extern crate byteorder;

pub const MSGPACK_VERSION : u32 = 5;

pub mod core;

pub use core::decode::{
    read_nil,
};

/// Temporary. Move to error.rs module.
pub use core::{Error, ReadError, MarkerError};
