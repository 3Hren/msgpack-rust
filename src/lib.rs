#![crate_name = "msgpack"]

#![unstable = "this library is still in rapid development"]

#![feature(core)]
#![cfg_attr(test, feature(test))]

extern crate byteorder;

pub const MSGPACK_VERSION : u32 = 5;

pub mod core;

/// Temporary. Move to error.rs module.
pub use core::{Error, ReadError, MarkerError};
