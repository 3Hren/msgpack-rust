use std::io::{self, Read};

use Value;

/// This type represents all possible errors that can occur when deserializing a value.
#[derive(Debug)]
pub enum Error {
    /// The MessagePack value had some syntatic error.
    Frame(ErrorCode),

    /// Some IO error occurred when serializing or deserializing a value.
    Io(io::Error),
}

/// Attempts to read bytes from the given reader and interpret them as a `Value`.
///
/// # Errors
///
/// This function will return `Error` on any I/O error while either reading or decoding a `Value`.
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
pub fn read_value<R>(rd: &mut R) -> Result<Value, Error>
    where R: Read
{
    let val = match try!(read_marker(rd)) {
        Marker::Null => Value::Nil,
        // Marker::True  => Value::Boolean(true),
        // Marker::False => Value::Boolean(false),
        // Marker::FixPos(val) => Value::Integer(Integer::U64(val as u64)),
        // Marker::FixNeg(val) => Value::Integer(Integer::I64(val as i64)),
        // Marker::U8  => Value::Integer(Integer::U64(try!(read_numeric_data::<R, u8>(rd))  as u64)),
        // Marker::U16 => Value::Integer(Integer::U64(try!(read_numeric_data::<R, u16>(rd)) as u64)),
        // Marker::U32 => Value::Integer(Integer::U64(try!(read_numeric_data::<R, u32>(rd)) as u64)),
        // Marker::U64 => Value::Integer(Integer::U64(try!(read_numeric_data(rd)))),
        // Marker::I8  => Value::Integer(Integer::I64(try!(read_numeric_data::<R, i8>(rd))  as i64)),
        // Marker::I16 => Value::Integer(Integer::I64(try!(read_numeric_data::<R, i16>(rd)) as i64)),
        // Marker::I32 => Value::Integer(Integer::I64(try!(read_numeric_data::<R, i32>(rd)) as i64)),
        // Marker::I64 => Value::Integer(Integer::I64(try!(read_numeric_data(rd)))),
        // Marker::F32 => Value::Float(Float::F32(try!(read_numeric_data(rd)))),
        // Marker::F64 => Value::Float(Float::F64(try!(read_numeric_data(rd)))),
        // Marker::FixStr(len) => {
        //     let len = len as u32;
        //     let res = try!(read_str(rd, len));
        //     Value::String(res)
        // }
        // Marker::Str8 => {
        //     let len = try!(read_numeric_data::<R, u8>(rd)) as u32;
        //     let res = try!(read_str(rd, len));
        //     Value::String(res)
        // }
        // Marker::Str16 => {
        //     let len = try!(read_numeric_data::<R, u16>(rd)) as u32;
        //     let res = try!(read_str(rd, len));
        //     Value::String(res)
        // }
        // Marker::Str32 => {
        //     let len = try!(read_numeric_data(rd));
        //     let res = try!(read_str(rd, len));
        //     Value::String(res)
        // }
        // Marker::FixArray(len) => {
        //     let len = len as usize;
        //     let vec = try!(read_array(rd, len));
        //     Value::Array(vec)
        // }
        // Marker::Array16 => {
        //     let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
        //     let vec = try!(read_array(rd, len));
        //     Value::Array(vec)
        // }
        // Marker::Array32 => {
        //     let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
        //     let vec = try!(read_array(rd, len));
        //     Value::Array(vec)
        // }
        // Marker::FixMap(len) => {
        //     let len = len as usize;
        //     let map = try!(read_map(rd, len));
        //     Value::Map(map)
        // }
        // Marker::Map16 => {
        //     let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
        //     let map = try!(read_map(rd, len));
        //     Value::Map(map)
        // }
        // Marker::Map32 => {
        //     let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
        //     let map = try!(read_map(rd, len));
        //     Value::Map(map)
        // }
        // Marker::Bin8 => {
        //     let len = try!(read_numeric_data::<R, u8>(rd)) as usize;
        //     let vec = try!(read_bin_data(rd, len));
        //     Value::Binary(vec)
        // }
        // Marker::Bin16 => {
        //     let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
        //     let vec = try!(read_bin_data(rd, len));
        //     Value::Binary(vec)
        // }
        // Marker::Bin32 => {
        //     let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
        //     let vec = try!(read_bin_data(rd, len));
        //     Value::Binary(vec)
        // }
        // Marker::FixExt1 => {
        //     let len = 1 as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::FixExt2 => {
        //     let len = 2 as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::FixExt4 => {
        //     let len = 4 as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::FixExt8 => {
        //     let len = 8 as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::FixExt16 => {
        //     let len = 16 as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::Ext8 => {
        //     let len = try!(read_numeric_data::<R, u8>(rd)) as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::Ext16 => {
        //     let len = try!(read_numeric_data::<R, u16>(rd)) as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::Ext32 => {
        //     let len = try!(read_numeric_data::<R, u32>(rd)) as usize;
        //     let (ty, vec) = try!(read_ext_body(rd, len));
        //     Value::Ext(ty, vec)
        // }
        // Marker::Reserved => return Err(Error::TypeMismatch(Marker::Reserved)),
        _ => unimplemented!(),
    };

    Ok(val)
}
