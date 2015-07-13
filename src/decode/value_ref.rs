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
    // Reading the marker involves either 1 byte read or nothing.
    let val = match read_marker(rd).unwrap() {
        Marker::Str8 => {
            let len = read_numeric_data::<R, u8>(rd).unwrap() as usize;
            let buf = rd.fill_buf().unwrap();
            let res = from_utf8(&buf[..len]).unwrap();
            ValueRef::String(res)
        }
        _ => unimplemented!(),
    };

    Ok(val)
}
