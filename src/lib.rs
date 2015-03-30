#![crate_name = "msgpack"]

#![unstable = "this library is still in rapid development"]

#![feature(core)]
#![feature(io)]
#![cfg_attr(test, feature(test))]

extern crate byteorder;

pub mod decode;

/// Temporary. Move to error.rs module.
pub use decode::{Error, ReadError, MarkerError};
