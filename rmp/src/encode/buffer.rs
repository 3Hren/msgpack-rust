//! Implementation of the [ByteBuf] type

use super::RmpWrite;

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