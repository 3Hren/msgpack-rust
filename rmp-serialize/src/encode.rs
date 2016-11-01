use std::io::Write;

use rustc_serialize;

use rmp::encode::{write_array_len, write_bool, write_f32, write_f64, write_map_len, write_nil,
                  write_sint, write_uint, write_str};

use rmp::encode::ValueWriteError;

pub type Error = ValueWriteError;

/// Represents MessagePack serialization implementation.
///
/// # Note
///
/// MessagePack has no specification about how to encode variant types. Thus we are free to do
/// whatever we want, so the given chose may be not ideal for you.
///
/// Every Rust variant value can be represented as a tuple of index and a value.
///
/// All instances of `ErrorKind::Interrupted` are handled by this function and the underlying
/// operation is retried.
// TODO: Docs. Examples, variant encoding policy.
pub struct Encoder<'a> {
    wr: &'a mut Write,
}

impl<'a> Encoder<'a> {
    /// Creates a new MessagePack encoder whose output will be written to the writer specified.
    pub fn new(wr: &'a mut Write) -> Encoder<'a> {
        Encoder {
            wr: wr,
        }
    }
}

impl<'a> rustc_serialize::Encoder for Encoder<'a> {
    type Error = Error;

    fn emit_nil(&mut self) -> Result<(), Error> {
        write_nil(&mut self.wr).map_err(|err| ValueWriteError::InvalidMarkerWrite(err))
    }

    fn emit_bool(&mut self, val: bool) -> Result<(), Error> {
        write_bool(&mut self.wr, val).map_err(|err| ValueWriteError::InvalidMarkerWrite(err))
    }

    fn emit_u8(&mut self, val: u8) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u16(&mut self, val: u16) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u32(&mut self, val: u32) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_u64(&mut self, val: u64) -> Result<(), Error> {
        try!(write_uint(&mut self.wr, val));
        Ok(())
    }

    fn emit_usize(&mut self, val: usize) -> Result<(), Error> {
        self.emit_u64(val as u64)
    }

    fn emit_i8(&mut self, val: i8) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i16(&mut self, val: i16) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i32(&mut self, val: i32) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_i64(&mut self, val: i64) -> Result<(), Error> {
        try!(write_sint(&mut self.wr, val));
        Ok(())
    }

    fn emit_isize(&mut self, val: isize) -> Result<(), Error> {
        self.emit_i64(val as i64)
    }

    fn emit_f32(&mut self, val: f32) -> Result<(), Error> {
        write_f32(&mut self.wr, val).map_err(From::from)
    }

    fn emit_f64(&mut self, val: f64) -> Result<(), Error> {
        write_f64(&mut self.wr, val).map_err(From::from)
    }

    // TODO: The implementation involves heap allocation and is unstable.
    fn emit_char(&mut self, val: char) -> Result<(), Error> {
        let mut buf = String::new();
        buf.push(val);
        self.emit_str(&buf)
    }

    fn emit_str(&mut self, val: &str) -> Result<(), Error> {
        write_str(&mut self.wr, val).map_err(From::from)
    }

    /// Encodes and attempts to write the enum value into the Write.
    ///
    /// Currently we encode variant types as a tuple of id with array of args, like: [id, [args...]]
    fn emit_enum<F>(&mut self, _name: &str, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        // Mark that we want to encode a variant type.
        try!(write_array_len(&mut self.wr, 2));

        // Delegate to the encoder of a concrete value.
        f(self)
    }

    /// Encodes and attempts to write a concrete variant value.
    fn emit_enum_variant<F>(&mut self, _name: &str, id: usize, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        // Encode a value position...
        try!(self.emit_usize(id));

        // ... and its arguments length.
        try!(write_array_len(&mut self.wr, len as u32));

        // Delegate to the encoder of a value args.
        f(self)
    }

    /// Encodes and attempts to write a concrete variant value arguments.
    fn emit_enum_variant_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_enum_struct_variant<F>(&mut self, _name: &str, _id: usize, _len: usize, _f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_enum_struct_variant_field<F>(&mut self, _name: &str, _idx: usize, _f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        unimplemented!()
    }

    fn emit_struct<F>(&mut self, _name: &str, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        self.emit_tuple(len, f)
    }

    fn emit_struct_field<F>(&mut self, _name: &str, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_array_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_tuple_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_tuple_struct<F>(&mut self, _name: &str, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        self.emit_tuple(len, f)
    }

    fn emit_tuple_struct_arg<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_option<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_option_none(&mut self) -> Result<(), Error> {
        self.emit_nil()
    }

    fn emit_option_some<F>(&mut self, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    // TODO: Check len, overflow is possible.
    fn emit_seq<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_array_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_seq_elt<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_map<F>(&mut self, len: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        try!(write_map_len(&mut self.wr, len as u32));
        f(self)
    }

    fn emit_map_elt_key<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> Result<(), Error>
        where F: FnOnce(&mut Self) -> Result<(), Error>
    {
        f(self)
    }
}
