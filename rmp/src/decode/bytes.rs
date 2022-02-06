//! Implementation of the [Bytes] type

use core::fmt::{Display, Formatter};
use crate::decode::RmpReadErr;
use super::RmpRead;

/// Indicates that an error occurred reading from [Bytes]
#[derive(Debug)]
#[non_exhaustive]
// NOTE: We can't use thiserror because of no_std :(
pub enum BytesReadError {
    /// Indicates that there were not enough bytes.
    ///
    /// Unfortunately this currently discards offset information (due to the implementation of [Bytes])
    InsufficientBytes {
        expected: usize,
        actual: usize,
    }
}

impl Display for BytesReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            BytesReadError::InsufficientBytes { expected, actual } => {
                write!(f, "Expected at least bytes {}, but only got {}", expected, actual)
            }
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for BytesReadError {}
impl RmpReadErr for BytesReadError {}

/// A wrapper around `&[u8]` to read more efficiently.
///
/// This has a specialized implementation of `RmpWrite`
/// and has error type [Infallible](core::convert::Infallible).
///
/// This has the additional benefit of working on `#[no_std]` (unlike the builtin Read trait)
///
/// See also [serde_bytes::Bytes](https://docs.rs/serde_bytes/0.11/serde_bytes/struct.Bytes.html)
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Bytes<'a> {
    bytes: &'a [u8],
}
impl<'a> Bytes<'a> {
    /// Wrap an existing bytes slice
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Self {
        Bytes { bytes }
    }
    /// Get a reference to this type as a slice of bytes (`&[u8]`)
    #[inline]
    pub fn as_slice(&self) -> &'a [u8] {
        &self.bytes
    }
}
impl AsRef<[u8]> for Bytes<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl<'a> From<&'a [u8]> for Bytes<'a> {
    #[inline]
    fn from(bytes: &'a [u8]) -> Self {
        Bytes { bytes }
    }
}

impl RmpRead for Bytes<'_> {
    type Error = BytesReadError;

    #[inline]
    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        if let Some((&first, newly_remaining)) = self.bytes.split_first() {
            self.bytes = newly_remaining;
            Ok(first)
        } else {
            Err(BytesReadError::InsufficientBytes {
                expected: 1,
                actual: 0
            })
        }
    }

    #[inline]
    fn read_exact_buf(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        if buf.len() < self.bytes.len() {
            let (src, newly_remaining) = self.bytes.split_at(buf.len());
            self.bytes = newly_remaining;
            buf.copy_from_slice(src);
            Ok(())
        } else {
            Err(BytesReadError::InsufficientBytes {
                expected: buf.len(),
                actual: self.bytes.len()
            })
        }
    }
}