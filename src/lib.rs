#![crate_name = "msgpack"]

#![unstable = "this library is still in rapid development"]

#![feature(core)]
#![feature(io)]

extern crate byteorder;

pub mod low;

pub use low::{Error, ReadError, MarkerError};
