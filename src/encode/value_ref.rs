//! This module is UNSTABLE, the reason is - recently added.

use std::convert::From;
use std::io::Write;

use super::super::value::ValueRef;
use super::*;

#[derive(Debug)]
pub struct Error(WriteError);

impl From<FixedValueWriteError> for Error {
    fn from(err: FixedValueWriteError) -> Error {
        match err {
            FixedValueWriteError(err) => Error(err)
        }
    }
}

pub fn write_value_ref<W>(wr: &mut W, val: &ValueRef) -> Result<(), Error>
    where W: Write
{
    match val {
        &ValueRef::Nil => try!(write_nil(wr)),
        &ValueRef::Boolean(val) => try!(write_bool(wr, val)),
        _ => unimplemented!(),
    }

    Ok(())
}
