use std::io::Write;

use rmp::encode::{write_nil, write_bool, write_uint, write_sint, write_f32, write_f64, write_str,
                  write_bin, write_array_len, write_map_len, write_ext_meta};

pub use rmp::encode::ValueWriteError as Error;

use Value;

/// Encodes and attempts to write the most efficient representation of the given Value.
///
/// # Note
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
pub fn write_value<W>(wr: &mut W, val: &Value) -> Result<(), Error>
    where W: Write
{
    match *val {
        Value::Nil => {
            write_nil(wr).map_err(|err| Error::InvalidMarkerWrite(err))?;
        }
        Value::Boolean(val) => {
            write_bool(wr, val).map_err(|err| Error::InvalidMarkerWrite(err))?;
        }
        Value::U64(val) => {
            write_uint(wr, val)?;
        }
        Value::I64(val) => {
            write_sint(wr, val)?;
        }
        Value::F32(val) => {
            write_f32(wr, val)?;
        }
        Value::F64(val) => {
            write_f64(wr, val)?;
        }
        Value::String(ref val) => {
            write_str(wr, &val)?;
        }
        Value::Binary(ref val) => {
            write_bin(wr, &val)?;
        }
        Value::Array(ref vec) => {
            write_array_len(wr, vec.len() as u32)?;
            for v in vec {
                write_value(wr, v)?;
            }
        }
        Value::Map(ref map) => {
            write_map_len(wr, map.len() as u32)?;
            for &(ref key, ref val) in map {
                write_value(wr, key)?;
                write_value(wr, val)?;
            }
        }
        Value::Ext(ty, ref data) => {
            write_ext_meta(wr, data.len() as u32, ty)?;
            wr.write_all(data).map_err(|err| Error::InvalidDataWrite(err))?;
        }
    }

    Ok(())
}
