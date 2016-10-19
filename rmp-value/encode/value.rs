// // pub mod value {
// //
// //     use std::convert::From;
// //     use std::fmt;
// //     use std::io::Write;
// //     use std::result::Result;
// //
// //     pub use super::super::value::{Integer, Float, Value};
// //
// //     use super::*;
// //
// //     #[derive(Debug)]
// //     pub enum Error {
// //         // TODO: Will be replaced with more concrete values.
// //         UnstableCommonError(String),
// //     }
// //
// //     impl ::std::error::Error for Error {
// //         fn description(&self) -> &str {
// //             match *self {
// //                 Error::UnstableCommonError(ref s) => s,
// //             }
// //         }
// //     }
// //
// //     impl fmt::Display for Error {
// //         fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// //             ::std::error::Error::description(self).fmt(f)
// //         }
// //     }
// //
// //
// //     impl From<FixedValueWriteError> for Error {
// //         fn from(err: FixedValueWriteError) -> Error {
// //             match err {
// //                 FixedValueWriteError(..) => {
// //                     Error::UnstableCommonError("fixed value error".to_string())
// //                 }
// //             }
// //         }
// //     }
// //
// //     impl From<ValueWriteError> for Error {
// //         fn from(_: ValueWriteError) -> Error {
// //             Error::UnstableCommonError("value error".to_string())
// //         }
// //     }
// //
// //     /// Encodes and attempts to write the most efficient representation of the given Value.
// //     ///
// //     /// # Note
// //     ///
// //     /// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
// //     /// operation is retried.
// //     // TODO: Docs. Examples.
// //     pub fn write_value<W>(wr: &mut W, val: &Value) -> Result<(), Error>
// //         where W: Write
// //     {
// //         match val {
// //             &Value::Nil => try!(write_nil(wr)),
// //             &Value::Boolean(val) => try!(write_bool(wr, val)),
// //             // TODO: Replace with generic write_int(...).
// //             &Value::Integer(Integer::U64(val)) => {
// //                 try!(write_uint(wr, val));
// //             }
// //             &Value::Integer(Integer::I64(val)) => {
// //                 try!(write_sint(wr, val));
// //             }
// //             // TODO: Replace with generic write_float(...).
// //             &Value::Float(Float::F32(val)) => try!(write_f32(wr, val)),
// //             &Value::Float(Float::F64(val)) => try!(write_f64(wr, val)),
// //             &Value::String(ref val) => {
// //                 try!(write_str(wr, &val));
// //             }
// //             &Value::Binary(ref val) => {
// //                 try!(write_bin(wr, &val));
// //             }
// //             &Value::Array(ref val) => {
// //                 try!(write_array_len(wr, val.len() as u32));
// //                 for item in val {
// //                     try!(write_value(wr, item));
// //                 }
// //             }
// //             &Value::Map(ref val) => {
// //                 try!(write_map_len(wr, val.len() as u32));
// //                 for &(ref key, ref val) in val {
// //                     try!(write_value(wr, key));
// //                     try!(write_value(wr, val));
// //                 }
// //             }
// //             &Value::Ext(ty, ref data) => {
// //                 try!(write_ext_meta(wr, data.len() as u32, ty));
// //                 try!(wr.write_all(data)
// //                     .map_err(|err| ValueWriteError::InvalidDataWrite(WriteError(err))));
// //             }
// //         }
// //
// //         Ok(())
// //     }
// //
// // } // mod value
// //
// // #[path = "encode/value_ref.rs"]
// // pub mod value_ref;


//! This module is UNSTABLE, the reason is - recently added.

use std::convert::From;
use std::error;
use std::fmt;
use std::io::Write;

// TODO: Includes cleanup is required.
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

/// Encodes and attempts to write the given non-owning ValueRef into the Write.
///
/// # Errors
///
/// This function returns Error with an underlying I/O error if unable to properly write entire
/// value. Interruption errors are handled internally by silent operation restarting.
///
/// # Examples
/// ```
/// use rmp::ValueRef;
/// use rmp::encode::value_ref::write_value_ref;
///
/// let mut buf = Vec::new();
/// let val = ValueRef::String("le message");
///
/// write_value_ref(&mut buf, &val).unwrap();
/// assert_eq!(vec![0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65], buf);
/// ```
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
    fn error_trait() {
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
