use std::error;
use std::io::{self, Read};
use std::fmt::{self, Display, Formatter};
use std::str::{Utf8Error, from_utf8};

use Marker;
use super::{read_marker, read_data_u8, read_data_u16, read_data_u32, Error, ValueReadError};

#[derive(Debug)]
pub enum DecodeStringError<'a> {
    InvalidMarkerRead(Error),
    InvalidDataRead(Error),
    TypeMismatch(Marker),
    /// The given buffer is not large enough to accumulate the specified amount of bytes.
    BufferSizeTooSmall(u32),
    InvalidUtf8(&'a [u8], Utf8Error),
}

impl<'a> error::Error for DecodeStringError<'a> {
    fn description(&self) -> &str {
        "error while decoding string"
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DecodeStringError::InvalidMarkerRead(ref err) |
            DecodeStringError::InvalidDataRead(ref err) => Some(err),
            DecodeStringError::TypeMismatch(..) |
            DecodeStringError::BufferSizeTooSmall(..) => None,
            DecodeStringError::InvalidUtf8(.., ref err) => Some(err),
        }
    }
}

impl<'a> Display for DecodeStringError<'a> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(f)
    }
}

impl<'a> From<ValueReadError> for DecodeStringError<'a> {
    fn from(err: ValueReadError) -> DecodeStringError<'a> {
        match err {
            ValueReadError::InvalidMarkerRead(err) => DecodeStringError::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => DecodeStringError::InvalidDataRead(err),
            ValueReadError::TypeMismatch(marker) => DecodeStringError::TypeMismatch(marker),
        }
    }
}

/// Attempts to read up to 9 bytes from the given reader and to decode them as a string `u32` size
/// value.
///
/// According to the MessagePack specification, the string format family stores an byte array in 1,
/// 2, 3, or 5 bytes of extra bytes in addition to the size of the byte array.
///
/// # Errors
///
/// This function will return `ValueReadError` on any I/O error while reading either the marker or
/// the data, except the EINTR, which is handled internally.
///
/// It also returns `ValueReadError::TypeMismatch` if the actual type is not equal with the
/// expected one, indicating you with the actual type.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
pub fn read_str_len<R: Read>(rd: &mut R) -> Result<u32, ValueReadError> {
    match try!(read_marker(rd)) {
        Marker::FixStr(size) => Ok(size as u32),
        Marker::Str8 => Ok(try!(read_data_u8(rd)) as u32),
        Marker::Str16 => Ok(try!(read_data_u16(rd)) as u32),
        Marker::Str32 => Ok(try!(read_data_u32(rd))),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Attempts to read a string data from the given reader and copy it to the buffer provided.
///
/// On success returns a borrowed string type, allowing to view the copyed bytes as properly utf-8
/// string.
/// According to the spec, the string's data must to be encoded using utf-8.
///
/// # Errors
///
/// Returns `Err` in the following cases:
///
///  - if any IO error (including unexpected EOF) occurs, while reading an `rd`, except the EINTR,
///    which is handled internally.
///  - if the `out` buffer size is not large enough to keep all the data copyed.
///  - if the data is not utf-8, with a description as to why the provided data is not utf-8 and
///    with a size of bytes actually copyed to be able to get them from `out`.
///
/// # Examples
/// ```
/// use rmp::decode::read_str;
///
/// let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
/// let mut out = [0u8; 16];
///
/// assert_eq!("le message", read_str(&mut &buf[..], &mut &mut out[..]).unwrap());
/// ```
///
/// # Unstable
///
/// This function is **unstable**, because it needs review.
///
/// # Note
///
/// This function will silently retry on every EINTR received from the underlying `Read` until
/// successful read.
// TODO: Stabilize. Mark error values for each error case (in docs).
pub fn read_str<'r, R>(rd: &mut R, mut buf: &'r mut [u8]) -> Result<&'r str, DecodeStringError<'r>>
    where R: Read
{
    let len = try!(read_str_len(rd));
    let ulen = len as usize;

    if buf.len() < ulen {
        return Err(DecodeStringError::BufferSizeTooSmall(len));
    }

    read_str_data(rd, len, &mut buf[0..ulen])
}

pub fn read_str_data<'r, R>(rd: &mut R,
                            len: u32,
                            buf: &'r mut [u8])
                            -> Result<&'r str, DecodeStringError<'r>>
    where R: Read
{
    debug_assert_eq!(len as usize, buf.len());

    // Trying to copy exact `len` bytes.
    match rd.read_exact(buf) {
        Ok(()) => {
            match from_utf8(buf) {
                Ok(decoded) => Ok(decoded),
                Err(err) => Err(DecodeStringError::InvalidUtf8(buf, err)),
            }
        }
        Err(err) => Err(DecodeStringError::InvalidDataRead(From::from(err))),
    }
}

/// Attempts to read and decode a string value from the reader, returning a borrowed slice from it.
///
// TODO: it is better to return &str; may panic on len mismatch; extend documentation.
// TODO: Also it's possible to implement all borrowing functions for all `BufRead` implementors.
// TODO: It's not necessary to use cursor, use slices instead.
// TODO: Candidate to be removed/replaced.
pub fn read_str_ref(rd: &[u8]) -> Result<&[u8], DecodeStringError> {
    let mut cur = io::Cursor::new(rd);
    let len = try!(read_str_len(&mut cur));
    let start = cur.position() as usize;
    Ok(&rd[start..start + len as usize])
}
