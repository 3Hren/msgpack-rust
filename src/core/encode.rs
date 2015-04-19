use std::io::Write;
use std::num::ToPrimitive;

use byteorder::{WriteBytesExt};

use super::{
    Error,
    Marker,
    Result,
};

fn write_marker<W>(wr: &mut W, marker: Marker) -> Result<()>
    where W: Write
{
    let byte = marker.to_u8().unwrap();

    match wr.write_u8(byte) {
        Ok(())   => Ok(()),
        Err(err) => Err(Error::InvalidMarkerWrite(From::from(err)))
    }
}

#[unstable(reason = "docs; stabilize Result variant; not sure about returning num of bytes written")]
pub fn write_nil<W>(wr: &mut W) -> Result<usize>
    where W: Write
{
    try!(write_marker(wr, Marker::Null));
    Ok(1)
}
