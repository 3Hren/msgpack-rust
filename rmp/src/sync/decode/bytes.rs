//! Implementation of the [Bytes] type

use crate::decode::RmpReadErr;
use super::RmpRead;
use crate::decode::{BytesReadError,Bytes};

impl RmpReadErr for BytesReadError {}



impl RmpRead for Bytes<'_> {
    type Error = BytesReadError;

    #[inline]
    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        if let Some((&first, newly_remaining)) = self.bytes.split_first() {
            self.bytes = newly_remaining;
            self.current_position += 1;
            Ok(first)
        } else {
            Err(BytesReadError::InsufficientBytes {
                expected: 1,
                actual: 0,
                position: self.current_position
            })
        }
    }

    #[inline]
    fn read_exact_buf(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let to_read = buf.len();
        if to_read <= self.bytes.len() {
            let (src, newly_remaining) = self.bytes.split_at(to_read);
            self.bytes = newly_remaining;
            self.current_position += to_read as u64;
            buf.copy_from_slice(src);
            Ok(())
        } else {
            Err(BytesReadError::InsufficientBytes {
                expected: to_read,
                actual: self.bytes.len(),
                position: self.current_position
            })
        }
    }
}

#[cfg(not(feature = "std"))]
impl<'a> RmpRead for &'a [u8] {
    type Error = BytesReadError;

    fn read_u8(&mut self) -> Result<u8, Self::Error> {
        if let Some((&first, newly_remaining)) = self.split_first() {
            *self = newly_remaining;
            Ok(first)
        } else {
            Err(BytesReadError::InsufficientBytes {
                expected: 1,
                actual: 0,
                position: 0
            })
        }
    }

    fn read_exact_buf(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let to_read = buf.len();
        if to_read <= self.len() {
            let (src, newly_remaining) = self.split_at(to_read);
            *self = newly_remaining;
            buf.copy_from_slice(src);
            Ok(())
        } else {
            Err(BytesReadError::InsufficientBytes {
                expected: to_read,
                actual: self.len(),
                position: 0
            })
        }
    }
}