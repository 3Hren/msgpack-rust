#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod decode;
pub mod encode;
mod errors;
mod marker;

pub use crate::marker::Marker;

/// Version of the MessagePack [spec](http://github.com/msgpack/msgpack/blob/master/spec.md).
pub const MSGPACK_VERSION: u32 = 5;

/// A type for holding Timestamp information
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Timestamp {
    size: u8,
    secs: i64,
    nsecs: u32,
}

impl Timestamp {
    /// Get the size of the Timestamp in bits
    #[inline]
    pub fn get_bitsize(self) -> u8 {
        self.size
    }

    /// Get the data to pass to chrono::DateTime::from_timestamp
    #[inline]
    pub fn into_timestamp(self) -> (i64, u32) {
        (
            self.secs,
            self.nsecs,
        )
    }

    /// Create a 32 bit timestamp using FixExt4
    #[inline]
    pub fn from_32(secs: u32) -> Self {
        Self { size: 32, secs: i64::from(secs), nsecs: 0 }
    }

    /// Create a 64 bit timestamp using FixExt8 with seconds and nanoseconds passed separately
    #[inline]
    pub fn from_64(secs: i64, nsecs: u32) -> Option<Self> {
        if secs < 0 || secs > 0x3_ffff_ffff || nsecs > 999_999_999 {
            None
        } else {
            Some(Self { size: 64, secs, nsecs })
        }
    }

    /// Create a 64 bit timestamp using FixExt8 from the combined 64 bit data
    #[inline]
    pub fn from_combined_64(data: u64) -> Option<Self> {
        // 30 bits fits in u32
        let nsecs = (data >> 34) as u32;
        if nsecs > 999_999_999 {
            return None
        }
        // 34 bits fits in i64
        let secs = (data & 0x3_ffff_ffff) as i64;

        Some(Self { size: 64, secs, nsecs })
    }

    /// Create a 96 bit timestamp using Ext8 (len=12)
    #[inline]
    pub fn from_96(secs: i64, nsecs: u32) -> Option<Self> {
        if nsecs > 999_999_999 {
            None
        } else {
            Some(Self { size: 96, secs, nsecs })
        }
    }

    /// Turns the data into a u128
    #[inline]
    pub fn into_u128(self) -> u128 {
        ((self.size as u128) << 96) |
        ((self.secs as u128) << 32) |
        (self.nsecs as u128)
    }

    /// Turns the data into a u128
    #[inline]
    pub fn from_u128(data: u128) -> Option<Self> {
        let nsecs = (data & u32::MAX as u128) as u32;
        let secs = ((data >> 32) & u64::MAX as u128) as i64;
        let size = ((data >> 96) & u8::MAX as u128) as u8;
        match size {
            32 => Some(Timestamp::from_32(secs as u32)),
            64 => Timestamp::from_64(secs, nsecs),
            96 => Timestamp::from_96(secs, nsecs),
            _ => None,
        }
    }
}
