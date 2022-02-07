//! Implementation of the [ByteBuf] type

use super::RmpWrite;
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use crate::encode::RmpWriteErr;

#[derive(Debug)]
pub struct FixedBufCapacityOverflow {
    needed_bytes: usize,
    remaining_bytes: usize,
    total_bytes: usize
}

#[cfg(feature = "std")]
impl std::error::Error for FixedBufCapacityOverflow {}
impl Display for FixedBufCapacityOverflow {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f, "Capacity overflow: Need to write {} bytes, but only {} remaining (total {})",
            self.needed_bytes, self.remaining_bytes, self.total_bytes
        )
    }
}
impl RmpWriteErr for FixedBufCapacityOverflow {}

/// EXPERIMENTAL: A version of [ByteBuf] with a fixed capacity.
///
/// This is a wrapper around `&mut [u8]`. It is intended primarily for `#[no_std]` targets,
/// where the regular `impl std::io::Write for &mut [u8]` impl is unavailable.
///
/// Unlike `ByteBuf`, it requires no allocation.
///
/// ## Safety
/// Unlike most other types in this crate, this uses unsafe code internally.
///
/// It is only line (avoiding a bounds check).
pub struct FixedByteBuf<'a> {
    buffer: &'a mut [u8],
    /// The position within the internal buffer.
    ///
    /// ## Safety
    /// Undefined behavior if `offset > self.buffer.len()`
    ///
    /// This is necessary to avoid bounds checks inside the innermost write method
    /// and work around the restrictions on mutable aliasing
    offset: usize
}

impl<'a> FixedByteBuf<'a> {
    /// Create a new [FixedByteBuf] from the specified buffer array.
    ///
    /// The capacity is fixed and cannot change.
    pub fn from_buf(buffer: &'a mut [u8]) -> Self {
        FixedByteBuf { buffer, offset: 0 }
    }
    /// Return the number of bytes that have currently been written.
    #[inline]
    pub fn len(&self) -> usize {
        self.offset
    }
    /// Return the (fixed) capacity of this buffer
    #[inline]
    pub fn total_capacity(&self) -> usize {
        self.buffer.len()
    }
    /// Return the remaining capacity, which is the number of bytes that can still be written.
    #[inline]
    pub fn remaining_capacity(&self) -> usize {
        self.total_capacity() - self.len()
    }
    #[inline]
    fn remaining_buffer(&mut self) -> &mut [u8] {
        debug_assert!(self.offset <= self.buffer.len());
        unsafe {
            // Optimizer doesn't like panics
            self.buffer.get_unchecked_mut(self.offset..)
        }
    }
    #[cold]
    #[inline]
    fn overflow_err(&self, needed_bytes: usize) -> FixedBufCapacityOverflow {
        FixedBufCapacityOverflow {
            total_bytes: self.total_capacity(),
            needed_bytes, remaining_bytes: self.remaining_capacity()
        }
    }
    /// Unwrap the originally underlying buffer
    ///
    /// This returns both the data that has been written,
    /// and the data that has not.
    #[inline]
    pub fn into_original_buffer(self) -> &'a mut [u8] {
        self.buffer
    }
}

impl RmpWrite for FixedByteBuf<'_> {
    type Error = FixedBufCapacityOverflow;

    #[inline]
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        let to_write = buf.len();
        let remaining = self.remaining_buffer();
        if to_write <= remaining.len() {
            remaining[..to_write].copy_from_slice(buf);
            self.offset += to_write;
            debug_assert!(self.offset <= self.buffer.len());
            Ok(())
        } else {
            Err(self.overflow_err(to_write))
        }
    }
}

/// A wrapper around `Vec<u8>` to serialize more efficiently.
///
/// This has a specialized implementation of `RmpWrite`.
///
/// This has the additional benefit of working on `#[no_std]`
///
/// See also [serde_bytes::ByteBuf](https://docs.rs/serde_bytes/0.11/serde_bytes/struct.ByteBuf.html)
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ByteBuf {
    bytes: Vec<u8>,
}
impl ByteBuf {
    /// Construct a new empty buffer
    #[inline]
    pub fn new() -> Self {
        ByteBuf { bytes: Vec::new() }
    }
    /// Construct a new buffer with the specified capacity
    ///
    /// See [Vec::with_capacity] for details
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        ByteBuf { bytes: Vec::with_capacity(capacity) }
    }
    /// Unwrap the underlying buffer of this vector
    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
    /// Wrap the specified vector as a [ByteBuf]
    #[inline]
    pub fn from_vec(bytes: Vec<u8>) -> Self {
        ByteBuf { bytes }
    }
    /// Get a reference to this type as a [Vec]
    #[inline]
    pub fn as_vec(&self) -> &Vec<u8> {
        &self.bytes
    }
    /// Get a mutable reference to this type as a [Vec]
    #[inline]
    pub fn as_mut_vec(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
    /// Get a reference to this type as a slice of bytes (`&[u8]`)
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}
impl AsRef<[u8]> for ByteBuf {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl AsRef<Vec<u8>> for ByteBuf {
    #[inline]
    fn as_ref(&self) -> &Vec<u8> {
        &self.bytes
    }
}
impl AsMut<Vec<u8>> for ByteBuf {
    #[inline]
    fn as_mut(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }
}
impl From<ByteBuf> for Vec<u8> {
    #[inline]
    fn from(buf: ByteBuf) -> Self {
        buf.bytes
    }
}
impl From<Vec<u8>> for ByteBuf {
    #[inline]
    fn from(bytes: Vec<u8>) -> Self {
        ByteBuf { bytes }
    }
}

impl RmpWrite for ByteBuf {
    type Error = core::convert::Infallible;

    #[inline]
    fn write_u8(&mut self, val: u8) -> Result<(), Self::Error> {
        self.bytes.push(val);
        Ok(())
    }

    #[inline]
    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        self.bytes.extend_from_slice(buf);
        Ok(())
    }
}