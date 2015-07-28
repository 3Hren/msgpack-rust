//! This module is UNSTABLE, the reason is - recently added.

use std::convert::From;
use std::io::Write;

use super::super::value::{Float, Integer, ValueRef};
use super::*;

// TODO: Make pub sometimes.
use super::{write_str, write_bin};

#[derive(Debug)]
pub struct Error(WriteError);

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
        _ => unimplemented!(),
    }

    Ok(())
}
