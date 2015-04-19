use std::io::Write;

use byteorder::{WriteBytesExt};

use super::{
    Error,
    Marker,
    Result,
    ToByte,
};

fn write_marker<W>(wr: &mut W, marker: Marker) -> Result<()>
    where W: Write
{
    match wr.write_u8(ToByte::to_byte(marker)) {
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
