use std::error;
use std::fmt::{self, Display, Formatter};
use std::io::Read;

use rustc_serialize;

use rmp::Marker;
use rmp::decode::{ValueReadError, NumValueReadError, DecodeStringError, read_nil, read_bool, read_int, read_f32,
                  read_f64, read_str_len, read_array_len, read_map_len};

/// Unstable: docs; incomplete
#[derive(Debug)]
pub enum Error {
    InvalidMarkerRead(::std::io::Error),
    InvalidDataRead(::std::io::Error),
    /// The actual value type isn't equal with the expected one.
    TypeMismatch(Marker),
    OutOfRange,
    LengthMismatch(u32),
    /// Uncategorized error.
    Uncategorized(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "error while decoding value"
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::Error::*;
        match *self {
            InvalidMarkerRead(ref err) => Some(err),
            InvalidDataRead(ref err) => Some(err),
            TypeMismatch(..) => None,
            OutOfRange => None,
            LengthMismatch(..) => None,
            Uncategorized(..) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        error::Error::description(self).fmt(f)
    }
}

impl From<ValueReadError> for Error {
    fn from(err: ValueReadError) -> Error {
        match err {
            ValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            ValueReadError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            ValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
        }
    }
}

impl From<NumValueReadError> for Error {
    fn from(err: NumValueReadError) -> Error {
        match err {
            NumValueReadError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            NumValueReadError::InvalidDataRead(err) => Error::InvalidDataRead(err),
            NumValueReadError::TypeMismatch(marker) => Error::TypeMismatch(marker),
            NumValueReadError::OutOfRange => Error::OutOfRange,
        }
    }
}

/// Unstable: docs; incomplete
impl<'a> From<DecodeStringError<'a>> for Error {
    fn from(err: DecodeStringError) -> Error {
        match err {
            DecodeStringError::InvalidMarkerRead(err) => Error::InvalidMarkerRead(err),
            DecodeStringError::InvalidDataRead(..) => {
                Error::Uncategorized("InvalidDataRead".to_string())
            }
            DecodeStringError::TypeMismatch(..) => Error::Uncategorized("TypeMismatch".to_string()),
            DecodeStringError::BufferSizeTooSmall(..) => {
                Error::Uncategorized("BufferSizeTooSmall".to_string())
            }
            DecodeStringError::InvalidUtf8(..) => Error::Uncategorized("InvalidUtf8".to_string()),
        }
    }
}

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
        Decoder { rd: rd }
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
impl<R: Read> rustc_serialize::Decoder for Decoder<R> {
    type Error = Error;

    fn read_nil(&mut self) -> Result<(), Error> {
        Ok(try!(read_nil(&mut self.rd)))
    }

    fn read_bool(&mut self) -> Result<bool, Error> {
        Ok(try!(read_bool(&mut self.rd)))
    }

    fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_u16(&mut self) -> Result<u16, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_u32(&mut self) -> Result<u32, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_u64(&mut self) -> Result<u64, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_usize(&mut self) -> Result<usize, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_i8(&mut self) -> Result<i8, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_i16(&mut self) -> Result<i16, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_i32(&mut self) -> Result<i32, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_i64(&mut self) -> Result<i64, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_isize(&mut self) -> Result<isize, Error> {
        Ok(try!(read_int(&mut self.rd)))
    }

    fn read_f32(&mut self) -> Result<f32, Error> {
        Ok(try!(read_f32(&mut self.rd)))
    }

    fn read_f64(&mut self) -> Result<f64, Error> {
        Ok(try!(read_f64(&mut self.rd)))
    }

    fn read_char(&mut self) -> Result<char, Error> {
        let mut res = try!(self.read_str());
        if res.len() == 1 {
            Ok(res.pop().unwrap())
        } else {
            Err(self.error("length mismatch"))
        }
    }

    fn read_str(&mut self) -> Result<String, Error> {
        let len = try!(read_str_len(&mut self.rd));

        let mut buf: Vec<u8> = vec![0u8; len as usize];

        try!(self.rd.read_exact(&mut buf).map_err(|err| Error::InvalidDataRead(err)));

        String::from_utf8(buf).map_err(|err| Error::Uncategorized(format!("{}", err)))
    }

    fn read_enum<T, F>(&mut self, _name: &str, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        let len = try!(read_array_len(&mut self.rd));
        if len == 2 {
            f(self)
        } else {
            Err(self.error("sequence length mismatch"))
        }
    }

    fn read_enum_variant<T, F>(&mut self, names: &[&str], mut f: F) -> Result<T, Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Error>
    {
        let id = try!(self.read_usize());

        if id < names.len() {
            try!(read_array_len(&mut self.rd));

            f(self, id)
        } else {
            Err(self.error("variant type overflow"))
        }
    }

    fn read_enum_variant_arg<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        f(self)
    }

    fn read_enum_struct_variant<T, F>(&mut self, names: &[&str], f: F) -> Result<T, Error>
        where F: FnMut(&mut Self, usize) -> Result<T, Error>
    {
        self.read_enum_variant(names, f)
    }

    fn read_enum_struct_variant_field<T, F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        f(self)
    }

    fn read_struct<T, F>(&mut self, _name: &str, len: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        self.read_tuple(len, f)
    }

    fn read_struct_field<T, F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        f(self)
    }

    fn read_tuple<T, F>(&mut self, len: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        let actual = try!(read_array_len(&mut self.rd));

        if len == actual as usize {
            f(self)
        } else {
            Err(Error::LengthMismatch(actual))
        }
    }

    // In case of MessagePack don't care about argument indexing.
    fn read_tuple_arg<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        f(self)
    }

    fn read_tuple_struct<T, F>(&mut self, _name: &str, _len: usize, _f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        unimplemented!()
    }
    fn read_tuple_struct_arg<T, F>(&mut self, _idx: usize, _f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        unimplemented!()
    }

    /// We treat Value::Null as None.
    fn read_option<T, F>(&mut self, mut f: F) -> Result<T, Error>
        where F: FnMut(&mut Self, bool) -> Result<T, Error>
    {
        // Primarily try to read optimisticly.
        match f(self, true) {
            Ok(val) => Ok(val),
            Err(Error::TypeMismatch(Marker::Null)) => f(self, false),
            Err(err) => Err(err),
        }
    }

    fn read_seq<T, F>(&mut self, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Error>
    {
        let len = try!(read_array_len(&mut self.rd)) as usize;

        f(self, len)
    }

    fn read_seq_elt<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        f(self)
    }

    fn read_map<T, F>(&mut self, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self, usize) -> Result<T, Error>
    {
        let len = try!(read_map_len(&mut self.rd)) as usize;

        f(self, len)
    }

    fn read_map_elt_key<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        f(self)
    }

    fn read_map_elt_val<T, F>(&mut self, _idx: usize, f: F) -> Result<T, Error>
        where F: FnOnce(&mut Self) -> Result<T, Error>
    {
        f(self)
    }

    fn error(&mut self, err: &str) -> Error {
        Error::Uncategorized(err.to_string())
    }
}
