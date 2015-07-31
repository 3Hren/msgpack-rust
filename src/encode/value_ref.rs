//! This module is UNSTABLE, the reason is - recently added.

use std::convert::From;
use std::error;
use std::fmt;
use std::io::Write;

use super::super::value::{Float, Integer, ValueRef};
use super::FixedValueWriteError;
use super::ValueWriteError;
use super::WriteError;
use super::write_array_len;
use super::write_bin;
use super::write_bool;
use super::write_ext_meta;
use super::write_f32;
use super::write_f64;
use super::write_map_len;
use super::write_nil;
use super::write_sint;
use super::write_str;
use super::write_uint;

#[derive(Debug)]
pub struct Error(WriteError);

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Error(ref err) => err.fmt(fmt),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            &Error(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            &Error(ref err) => err.cause(),
        }
    }
}

impl From<FixedValueWriteError> for Error {
    fn from(err: FixedValueWriteError) -> Error {
        match err {
            FixedValueWriteError(err) => Error(err)
        }
    }
}

impl From<ValueWriteError> for Error {
    fn from(err: ValueWriteError) -> Error {
        match err {
            ValueWriteError::InvalidMarkerWrite(err) => Error(err),
            ValueWriteError::InvalidDataWrite(err) => Error(err)
        }
    }
}

pub fn write_value_ref<W>(wr: &mut W, val: &ValueRef) -> Result<(), Error>
    where W: Write
{
    match val {
        &ValueRef::Nil => try!(write_nil(wr)),
        &ValueRef::Boolean(val) => try!(write_bool(wr, val)),
        &ValueRef::Integer(Integer::U64(val)) => {
            try!(write_uint(wr, val));
        }
        &ValueRef::Integer(Integer::I64(val)) => {
            try!(write_sint(wr, val));
        }
        // TODO: Replace with generic write_float(...).
        &ValueRef::Float(Float::F32(val)) => try!(write_f32(wr, val)),
        &ValueRef::Float(Float::F64(val)) => try!(write_f64(wr, val)),
        &ValueRef::String(val) => {
            try!(write_str(wr, val));
        }
        &ValueRef::Binary(val) => {
            try!(write_bin(wr, val));
        }
        &ValueRef::Array(ref val) => {
            let len = val.len() as u32;

            try!(write_array_len(wr, len));

            for item in val {
                try!(write_value_ref(wr, item));
            }
        }
        &ValueRef::Map(ref val) => {
            let len = val.len() as u32;

            try!(write_map_len(wr, len));

            for &(ref key, ref val) in val {
                try!(write_value_ref(wr, key));
                try!(write_value_ref(wr, val));
            }
        }
        &ValueRef::Ext(ty, data) => {
            try!(write_ext_meta(wr, data.len() as u32, ty));
            try!(wr.write_all(data).map_err(|err| ValueWriteError::InvalidDataWrite(WriteError(err))));
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::error;
    use std::io;
    use super::super::WriteError;
    use super::Error;

    #[test]
    fn display_trait() {
        let err = Error(WriteError(io::Error::new(io::ErrorKind::Other, "unexpected EOF")));

        assert_eq!("err: error while writing MessagePack'ed value", format!("err: {}", err));
    }

    #[test]
    fn error_trait_description() {
        // Delegates to I/O Error.
        let err = Error(WriteError(io::Error::new(io::ErrorKind::Other, "unexpected EOF")));

        fn test<E: error::Error>(err: E) {
            assert_eq!("error while writing MessagePack'ed value", err.description());
            assert!(err.cause().is_some());
            assert_eq!("unexpected EOF", err.cause().unwrap().description());
        }

        test(err);
    }
}
