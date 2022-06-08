use core::str::Utf8Error;
use crate::decode::{RmpReadErr};
use crate::Marker;

#[derive(Debug)]
#[allow(deprecated)] // Only for compatibility
pub enum DecodeStringError<'a, E: RmpReadErr = super::Error> {
    InvalidMarkerRead(E),
    InvalidDataRead(E),
    TypeMismatch(Marker),
    /// The given buffer is not large enough to accumulate the specified amount of bytes.
    BufferSizeTooSmall(u32),
    InvalidUtf8(&'a [u8], Utf8Error),
}

