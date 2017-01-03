use std;
// use std::convert::From;
// use std::error;
// use std::fmt;
use std::io::{self, Cursor, ErrorKind, Read};
use std::str::{from_utf8, Utf8Error};

use rmp::Marker;
use rmp::decode::{read_marker, read_data_u8, read_data_u16, read_data_u32, read_data_u64,
                  read_data_i8, read_data_i16, read_data_i32, read_data_i64, read_data_f32,
                  read_data_f64, MarkerReadError, ValueReadError};

use ValueRef;

#[derive(Debug)]
pub enum Error<'r> {
    /// Failed to read the type marker value.
    InvalidMarkerRead(io::Error),
    /// Failed to read packed non-marker data.
    InvalidDataRead(io::Error),
    /// Decoded value type isn't equal with the expected one.
    TypeMismatch(Marker),
    /// Failed to interpret a byte slice as a UTF-8 string.
    ///
    /// Contains untouched bytearray with the underlying decoding error.
    InvalidUtf8(&'r [u8], Utf8Error),
}

// impl<'r> error::Error for Error<'r> {
//     fn description(&self) -> &str {
//         match self {
//             &Error::InvalidMarkerRead(..) => "failed to read the type marker value",
//             &Error::InvalidLengthRead(..) => "failed to read string/array/map size",
//             &Error::InvalidDataRead(..) => "failed to read packed non-marker data",
//             &Error::InvalidLengthSize => "failed to cast the length read to machine size",
//             &Error::InvalidUtf8(..) => "failed to interpret a byte slice as a UTF-8 string",
//             &Error::InvalidExtTypeRead(..) => "failed to read ext type",
//             &Error::TypeMismatch => "using Reserved type found",
//         }
//     }
//
//     fn cause(&self) -> Option<&error::Error> {
//         match self {
//             &Error::InvalidMarkerRead(ref err) => Some(err),
//             &Error::InvalidLengthRead(ref err) => Some(err),
//             &Error::InvalidDataRead(ref err) => Some(err),
//             &Error::InvalidLengthSize => None,
//             &Error::InvalidUtf8(_, ref err) => Some(err),
//             &Error::InvalidExtTypeRead(ref err) => Some(err),
//             &Error::TypeMismatch => None,
//         }
//     }
// }
//
// impl<'r> fmt::Display for Error<'r> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         use std::error::Error;
//         self.description().fmt(f)
//     }
// }
//

impl<'r> From<MarkerReadError> for Error<'r> {
    fn from(err: MarkerReadError) -> Error<'r> {
        Error::InvalidMarkerRead(err.0)
    }
}

impl<'r> From<ValueReadError> for Error<'r> {
    fn from(err: ValueReadError) -> Error<'r> {
        match err {
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            ValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
    }
}

// fn read_len<R, D>(rd: &mut R) -> Result<D, ReadError>
//     where R: Read,
//           D: BigEndianRead
// {
//     D::read(rd).map_err(From::from)
// }
//
// fn read_num<'a, R, D>(mut rd: &mut R) -> Result<D, Error<'a>>
//     where R: BorrowRead<'a>,
//           D: BigEndianRead
// {
//     D::read(&mut rd).map_err(|err| Error::InvalidDataRead(From::from(err)))
// }

fn read_str_data<'a, R>(rd: &mut R, len: usize) -> Result<&'a str, Error<'a>>
    where R: BorrowRead<'a>
{
    let buf = read_bin_data(rd, len)?;
    from_utf8(buf).map_err(|err| Error::InvalidUtf8(buf, err))
}

fn read_bin_data<'a, R>(rd: &mut R, len: usize) -> Result<&'a [u8], Error<'a>>
    where R: BorrowRead<'a>
{
    let buf = rd.fill_buf();

    if len > buf.len() {
        return Err(Error::InvalidDataRead(io::Error::new(ErrorKind::UnexpectedEof, "unexpected EOF")));
    }

    // Take a slice.
    let buf = &buf[..len];
    rd.consume(len);

    Ok(buf)
}

fn read_ext_body<'a, R>(rd: &mut R, len: usize) -> Result<(i8, &'a [u8]), Error<'a>>
    where R: BorrowRead<'a>
{
    let ty = read_data_i8(rd)?;
    let buf = read_bin_data(rd, len)?;

    Ok((ty, buf))
}

fn read_array_data<'a, R>(rd: &mut R, mut len: usize) -> Result<Vec<ValueRef<'a>>, Error<'a>>
    where R: BorrowRead<'a>
{
    let mut vec = Vec::with_capacity(len);

    while len > 0 {
        vec.push(read_value_ref(rd)?);
        len -= 1;
    }

    Ok(vec)
}

fn read_map_data<'a, R>(rd: &mut R, mut len: usize) -> Result<Vec<(ValueRef<'a>, ValueRef<'a>)>, Error<'a>>
    where R: BorrowRead<'a>
{
    let mut vec = Vec::with_capacity(len);

    while len > 0 {
        vec.push((read_value_ref(rd)?, read_value_ref(rd)?));
        len -= 1;
    }

    Ok(vec)
}

/// A BorrowRead is a type of Reader which has an internal buffer.
///
/// This magic trait acts like a standard BufRead but unlike the standard this has an explicit
/// internal buffer lifetime, which allows to borrow from underlying buffer while consuming bytes.
pub trait BorrowRead<'a>: Read {
    /// Returns the buffer contents.
    ///
    /// This function is a lower-level call. It needs to be paired with the consume method to
    /// function properly. When calling this method, none of the contents will be "read" in the
    /// sense that later calling read may return the same contents. As such, consume must be called
    /// with the number of bytes that are consumed from this buffer to ensure that the bytes are
    /// never returned twice.
    ///
    /// An empty buffer returned indicates that the stream has reached EOF.
    fn fill_buf(&self) -> &'a [u8];

    /// Tells this buffer that len bytes have been consumed from the buffer, so they should no
    /// longer be returned in calls to read.
    fn consume(&mut self, len: usize);
}

impl<'a> BorrowRead<'a> for &'a [u8] {
    fn fill_buf(&self) -> &'a [u8] {
        self
    }

    fn consume(&mut self, len: usize) {
        *self = &(*self)[len..];
    }
}

/// Useful when you want to know how much bytes has been consumed during ValueRef decoding.
impl<'a> BorrowRead<'a> for Cursor<&'a [u8]> {
    fn fill_buf(&self) -> &'a [u8] {
        let len = std::cmp::min(self.position(), self.get_ref().len() as u64);
        &self.get_ref()[len as usize..]
    }

    fn consume(&mut self, len: usize) {
        let pos = self.position();
        self.set_position(pos + len as u64);
    }
}

/// Attempts to read the data from the given reader until either a complete MessagePack value
/// decoded or an error detected.
///
/// Returns either a non-owning `ValueRef`, which borrows the buffer from the given reader or an
/// error.
///
/// The reader should meet the requirement of a special `BorrowRead` trait, which allows to mutate
/// itself but permits to mutate the buffer it contains. It allows to perform a completely
/// zero-copy reading without a data loss fear in case of an error.
///
/// Currently only two types fit in this requirement: `&[u8]` and `Cursor<&[u8]>`. Using Cursor is
/// helpful, when you need to know how exactly many bytes the decoded ValueRef consumes. A `Vec<u8>`
/// type doesn't fit in the `BorrowRead` requirement, because its mut reference can mutate the
/// underlying buffer - use `Vec::as_slice()` if you need to decode a value from the vector.
///
/// # Errors
///
/// Returns an `Error` value if unable to continue the decoding operation either because of read
/// failure or any other circumstances. See `Error` documentation for more information.
///
/// # Examples
/// ```
/// use rmpv::ValueRef;
/// use rmpv::decode::value_ref::read_value_ref;
///
/// let buf = [0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65];
/// let mut rd = &buf[..];
///
/// assert_eq!(ValueRef::String("le message"), read_value_ref(&mut rd).unwrap());
/// ```
pub fn read_value_ref<'a, R>(rd: &mut R) -> Result<ValueRef<'a>, Error<'a>>
    where R: BorrowRead<'a>
{
    let mut rd = rd;

    // Reading the marker involves either 1 byte read or nothing. On success consumes strictly
    // 1 byte from the `rd`.
    let val = match read_marker(rd)? {
        Marker::Null => ValueRef::Nil,
        Marker::True => ValueRef::Boolean(true),
        Marker::False => ValueRef::Boolean(false),
        Marker::FixPos(val) => ValueRef::U64(val as u64),
        Marker::FixNeg(val) => ValueRef::I64(val as i64),
        Marker::U8 => ValueRef::U64(read_data_u8(rd)? as u64),
        Marker::U16 => ValueRef::U64(read_data_u16(rd)? as u64),
        Marker::U32 => ValueRef::U64(read_data_u32(rd)? as u64),
        Marker::U64 => ValueRef::U64(read_data_u64(rd)?),
        Marker::I8 => ValueRef::I64(read_data_i8(rd)? as i64),
        Marker::I16 => ValueRef::I64(read_data_i16(rd)? as i64),
        Marker::I32 => ValueRef::I64(read_data_i32(rd)? as i64),
        Marker::I64 => ValueRef::I64(read_data_i64(rd)?),
        Marker::F32 => ValueRef::F32(read_data_f32(rd)?),
        Marker::F64 => ValueRef::F64(read_data_f64(rd)?),
        Marker::FixStr(len) => {
            let res = read_str_data(rd, len as usize)?;
            ValueRef::String(res)
        }
        Marker::Str8 => {
            let len = read_data_u8(rd)?;
            let res = read_str_data(rd, len as usize)?;
            ValueRef::String(res)
        }
        Marker::Str16 => {
            let len = read_data_u16(rd)?;
            let res = read_str_data(rd, len as usize)?;
            ValueRef::String(res)
        }
        Marker::Str32 => {
            let len = read_data_u32(rd)?;
            let res = read_str_data(rd, len as usize)?;
            ValueRef::String(res)
        }
        Marker::Bin8 => {
            let len = read_data_u8(rd)?;
            let res = read_bin_data(rd, len as usize)?;
            ValueRef::Binary(res)
        }
        Marker::Bin16 => {
            let len = read_data_u16(rd)?;
            let res = read_bin_data(rd, len as usize)?;
            ValueRef::Binary(res)
        }
        Marker::Bin32 => {
            let len = read_data_u32(rd)?;
            let res = read_bin_data(rd, len as usize)?;
            ValueRef::Binary(res)
        }
        Marker::FixArray(len) => {
            let vec = read_array_data(rd, len as usize)?;
            ValueRef::Array(vec)
        }
        Marker::Array16 => {
            let len = read_data_u16(rd)?;
            let vec = read_array_data(rd, len as usize)?;
            ValueRef::Array(vec)
        }
        Marker::Array32 => {
            let len = read_data_u32(rd)?;
            let vec = read_array_data(rd, len as usize)?;
            ValueRef::Array(vec)
        }
        Marker::FixMap(len) => {
            let map = read_map_data(rd, len as usize)?;
            ValueRef::Map(map)
        }
        Marker::Map16 => {
            let len = read_data_u16(rd)?;
            let map = read_map_data(rd, len as usize)?;
            ValueRef::Map(map)
        }
        Marker::Map32 => {
            let len = read_data_u32(rd)?;
            let map = read_map_data(rd, len as usize)?;
            ValueRef::Map(map)
        }
        Marker::FixExt1 => {
            let len = 1;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::FixExt2 => {
            let len = 2;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::FixExt4 => {
            let len = 4;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::FixExt8 => {
            let len = 8;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::FixExt16 => {
            let len = 16;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::Ext8 => {
            let len = read_data_u8(rd)?;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::Ext16 => {
            let len = read_data_u16(rd)?;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::Ext32 => {
            let len = read_data_u32(rd)?;
            let (ty, vec) = read_ext_body(rd, len as usize)?;
            ValueRef::Ext(ty, vec)
        }
        Marker::Reserved => return Err(Error::TypeMismatch(Marker::Reserved)),
    };

    Ok(val)
}
