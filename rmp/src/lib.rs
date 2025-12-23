#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::match_same_arms)]

extern crate alloc;

pub mod decode;
pub mod encode;
mod errors;
mod marker;

pub use crate::marker::Marker;

/// Version of the MessagePack [spec](http://github.com/msgpack/msgpack/blob/master/spec.md).
pub const MSGPACK_VERSION: u32 = 5;
