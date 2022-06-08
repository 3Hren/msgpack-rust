use core::fmt::{Display, Formatter};

/// Indicates that an error occurred reading from [Bytes]
#[derive(Debug)]
#[non_exhaustive]
// NOTE: We can't use thiserror because of no_std :(
pub enum BytesReadError {
    /// Indicates that there were not enough bytes.
    InsufficientBytes {
        expected: usize,
        actual: usize,
        position: u64
    }
}

impl Display for BytesReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match *self {
            BytesReadError::InsufficientBytes { expected, actual, position } => {
                write!(f, "Expected at least bytes {}, but only got {} (pos {})", expected, actual, position)
            }
        }
    }
}
#[cfg(feature = "std")]
impl std::error::Error for BytesReadError {}

/// A wrapper around `&[u8]` to read more efficiently.
///
/// This has a specialized implementation of `RmpWrite`
/// and has error type [Infallible](core::convert::Infallible).
///
/// This has the additional benefit of working on `#[no_std]` (unlike the builtin Read trait)
///
/// See also [serde_bytes::Bytes](https://docs.rs/serde_bytes/0.11/serde_bytes/struct.Bytes.html)
///
/// Unlike a plain `&[u8]` this also tracks an internal offset in the input (See [Self::position]).
///
/// This is used for (limited) compatibility with [std::io::Cursor]. Unlike a [Cursor](std::io::Cursor) it does
/// not support mark/reset.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct Bytes<'a> {
    /// The internal position of the input buffer.
    ///
    /// This is not required for correctness.
    /// It is only used for error reporting (and to implement [Self::position])
    pub(crate)current_position: u64,
    pub(crate)bytes: &'a [u8],
}
impl<'a> Bytes<'a> {
    /// Wrap an existing bytes slice.
    ///
    /// This sets the internal position to zero.
    #[inline]
    pub fn new(bytes: &'a [u8]) -> Self {
        Bytes { bytes, current_position: 0 }
    }
    /// Get a reference to the remaining bytes in the buffer.
    #[inline]
    pub fn remaining_slice(&self) -> &'a [u8] {
        self.bytes
    }
    /// Return the position of the input buffer.
    ///
    /// This is not required for correctness, it only exists to help mimic
    /// [Cursor::position](std::io::Cursor::position)
    #[inline]
    pub fn position(&self) -> u64 {
        self.current_position
    }
}
impl<'a> From<&'a [u8]> for Bytes<'a> {
    #[inline]
    fn from(bytes: &'a [u8]) -> Self {
        Bytes { bytes, current_position: 0 }
    }
}