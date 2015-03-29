#![crate_name = "msgpack"]

#![unstable = "this library is still in rapid development"]

#![feature(core)]
#![feature(io)]
#![cfg_attr(test, feature(test))]

extern crate byteorder;

pub mod low;

pub use low::{Error, ReadError, MarkerError};
