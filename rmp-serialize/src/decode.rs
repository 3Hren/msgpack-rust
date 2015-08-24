use std::convert::From;
use std::fmt;
use std::io::Read;
use std::result;

use serialize;

use rmp::Marker;
use rmp::decode::{
    ReadError,
    FixedValueReadError,
    ValueReadError,
    DecodeStringError,
    read_nil,
    read_bool,
    read_u8_fit,
    read_u16_fit,
    read_u32_fit,
    read_u64_fit,
    read_i8_fit,
    read_i16_fit,
    read_i32_fit,
    read_i64_fit,
    read_f32,
    read_f64,
    read_str_len,
    read_str_data,
    read_array_size,
    read_map_size,
};

/// Unstable: docs; incomplete
#[derive(Debug)]
pub enum Error {
    /// The actual value type isn't equal with the expected one.
    TypeMismatch(Marker),
    InvalidMarkerRead(ReadError),
    InvalidDataRead(ReadError),
    LengthMismatch(u32),
    /// Uncategorized error.
    Uncategorized(String),
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str { "error while decoding value" }

    fn cause(&self) -> Option<&::std::error::Error> {
        use self::Error::*;
        match *self {
            TypeMismatch(_) => None,
            InvalidMarkerRead(ref err) => Some(err),
            InvalidDataRead(ref err) => Some(err),
            LengthMismatch(_) => None,
            Uncategorized(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        ::std::error::Error::description(self).fmt(f)
    }
}

impl From<FixedValueReadError> for Error {
    fn from(err: FixedValueReadError) -> Error {
        match err {
            FixedValueReadError::UnexpectedEOF => Error::InvalidMarkerRead(ReadError::UnexpectedEOF),
            FixedValueReadError::Io(err) => Error::InvalidMarkerRead(ReadError::Io(err)),
            FixedValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Error {
        match err {
            ValueReadError::TypeMismatch(marker)   => Error::TypeMismatch(marker),
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err)   => Error::InvalidDataRead(err),
        }
    }
}

/// Unstable: docs; incomplete
impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(..) => Error::Uncategorized("InvalidDataRead".to_string()),
            DecodeStringError::TypeMismatch(..) => Error::Uncategorized("TypeMismatch".to_string()),
            DecodeStringError::BufferSizeTooSmall(..) => Error::Uncategorized("BufferSizeTooSmall".to_string()),
            DecodeStringError::InvalidDataCopy(..) => Error::Uncategorized("InvalidDataCopy".to_string()),
            DecodeStringError::InvalidUtf8(..) => Error::Uncategorized("InvalidUtf8".to_string()),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples.
pub struct Decoder<R: Read> {
    rd: R,
}

impl<R: Read> Decoder<R> {
    // TODO: Docs.
    pub fn new(rd: R) -> Decoder<R> {
        Decoder {
            rd: rd
        }
    }

    /// Gets a reference to the underlying reader in this decoder.
    pub fn get_ref(&self) -> &R {
        &self.rd
    }

    /// Gets a mutable reference to the underlying reader in this decoder.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.rd
    }

    /// Consumes this decoder returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.rd
    }
}

/// Unstable: docs; examples; incomplete
impl<R: Read> serialize::Decoder for Decoder<R> {
    type Error = Error;

    fn read_nil(&mut self) -> Result<()> {
        Ok(try!(read_nil(&mut self.rd)))
    }

    fn read_bool(&mut self) -> Result<bool> {
        Ok(try!(read_bool(&mut self.rd)))
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(try!(read_u8_fit(&mut self.rd)))
    }

    fn read_u16(&mut self) -> Result<u16> {
        Ok(try!(read_u16_fit(&mut self.rd)))
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(try!(read_u32_fit(&mut self.rd)))
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(try!(read_u64_fit(&mut self.rd)))
    }

    /// TODO: Doesn't look safe.
    fn read_usize(&mut self) -> Result<usize> {
        let v = try!(self.read_u64());
        Ok(v as usize)
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(try!(read_i8_fit(&mut self.rd)))
    }

    fn read_i16(&mut self) -> Result<i16> {
        Ok(try!(read_i16_fit(&mut self.rd)))
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(try!(read_i32_fit(&mut self.rd)))
    }

    fn read_i64(&mut self) -> Result<i64> {
        Ok(try!(read_i64_fit(&mut self.rd)))
    }

    /// TODO: Doesn't look safe.
    fn read_isize(&mut self) -> Result<isize> {
        Ok(try!(self.read_i64()) as isize)
    }

    fn read_f32(&mut self) -> Result<f32> {
        Ok(try!(read_f32(&mut self.rd)))
    }

    fn read_f64(&mut self) -> Result<f64> {
        Ok(try!(read_f64(&mut self.rd)))
    }

    fn read_char(&mut self) -> Result<char> {
        let mut res = try!(self.read_str());
        if res.len() == 1 {
            Ok(res.pop().unwrap())
        } else {
            Err(self.error("length mismatch"))
        }
    }

    fn read_str(&mut self) -> Result<String> {
        let len = try!(read_str_len(&mut self.rd));

        let mut buf: Vec<u8> = (0..len).map(|_| 0u8).collect();

        Ok(try!(read_str_data(&mut self.rd, len, &mut buf[..])).to_string())
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        let len = try!(read_array_size(&mut self.rd));
        if len == 2 {
            f(self)
        } else {
            Err(self.error("sequence length mismatch"))
        }
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str], mut f: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        let id = try!(self.read_usize());

        if id < names.len() {
            try!(read_array_size(&mut self.rd));

            f(self, id)
        } else {
            Err(self.error("variant type overflow"))
        }
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T>
        where F: FnMut(&mut Self, usize) -> Result<T>
    {
        self.read_enum_variant(names, f)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_struct<T, F>(&mut self, _name: &str, len: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        self.read_tuple(len, f)
    }

    fn read_struct_field<T, F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, len: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        let actual = try!(read_array_size(&mut self.rd));

        if len == actual as usize {
            f(self)
        } else {
            Err(Error::LengthMismatch(actual))
        }
    }

    // In case of MessagePack don't care about argument indexing.
    fn read_tuple_arg<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _name: &str, _len: usize, _f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T> { unimplemented!() }
    fn read_tuple_struct_arg<T, F>(&mut self, _idx: usize, _f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T> { unimplemented!() }

    /// We treat Value::Null as None.
    fn read_option<T, F>(&mut self, mut f: F) -> Result<T>
        where F: FnMut(&mut Self, bool) -> Result<T>
    {
        // Primarily try to read optimisticly.
        match f(self, true) {
            Ok(val) => Ok(val),
            Err(Error::TypeMismatch(Marker::Null)) => f(self, false),
            Err(err) => Err(err)
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        let len = try!(read_array_size(&mut self.rd)) as usize;

        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> Result<T>
        where F: FnOnce(&mut Self, usize) -> Result<T>
    {
        let len = try!(read_map_size(&mut self.rd)) as usize;

        f(self, len)
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T>
        where F: FnOnce(&mut Self) -> Result<T>
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> Error {
        Error::Uncategorized(err.to_string())
    }
}
