use std::io::BufRead;
use std::str::from_utf8;

use super::{read_marker, read_numeric_data};
use super::super::init::Marker;
use super::super::value::ValueRef;

// TODO: Display trait.
#[derive(Debug)]
pub struct Error;

pub fn read_value_ref<R>(rd: &mut R) -> Result<ValueRef, Error>
    where R: BufRead
{
    let mut buf = rd.fill_buf().unwrap(); // TODO: May fail.

    // Reading the marker involves either 1 byte read or nothing. Consumes 1 byte from `buf`, not
    // from `rd`.
    let val = match read_marker(&mut buf).unwrap() { // TODO: May fail (IO).
        Marker::Str8 => {
            let len = read_numeric_data::<&[u8], u8>(&mut buf).unwrap(); // TODO: May fail (IO).
            let len = len as usize; // TODO: May panic.
            // TODO: Check buffer length.
            let res = from_utf8(&buf[..len]).unwrap(); // TODO: May fail (not UTF-8), return &[u8] otherwise.
            ValueRef::String(res)
        }
        _ => unimplemented!(),
    };

    Ok(val)
}
