use serde::Serialize;
use rmp_serde::Serializer;

#[test]
fn pass_struct() {
    let mut buf = Vec::new();

    #[derive(Serialize)]
    struct Struct {
        f1: u32,
        f2: u32,
    }

    let val = Struct {
        f1: 42,
        f2: 100500,
    };
    val.serialize(&mut Serializer::new(&mut buf)).ok().unwrap();

    // Expect: [42, 100500].
    assert_eq!(vec![0x92, 0x2a, 0xce, 0x0, 0x1, 0x88, 0x94], buf);
}

#[test]
fn pass_struct_empty() {
    let mut buf = Vec::new();

    #[derive(Serialize)]
    struct Struct;

    Struct.serialize(&mut Serializer::new(&mut buf)).ok().unwrap();

    // Expect: [].
    assert_eq!(vec![0x90], buf);
}

#[test]
fn pass_struct_map() {
    // TODO: Refactor: add builder, make serializer configurable.
    use std::io::Write;
    use rmp::Marker;
    use rmp::encode::{ValueWriteError, write_map_len, write_str};
    use rmp_serde::encode::VariantWriter;

    struct StructMapWriter;

    impl VariantWriter for StructMapWriter {
        fn write_struct_len<W: Write>(&self, wr: &mut W, len: u32) ->
            ::std::result::Result<Marker, ValueWriteError>
        {
            write_map_len(wr, len)
        }

        fn write_field_name<W: Write>(&self, wr: &mut W, key: &str) ->
            ::std::result::Result<(), ValueWriteError>
        {
            write_str(wr, key)
        }
    }

    #[derive(Debug, PartialEq, Serialize)]
    struct Custom<'a> {
        et: &'a str,
        le: u8,
        shit: u8,
    }

    let mut buf = [0x00; 20];

    let val = Custom {
        et: "voila",
        le: 0,
        shit: 1,
    };
    val.serialize(&mut Serializer::with(&mut &mut buf[..], StructMapWriter)).ok().unwrap();

    // Expect: {"et": "voila", "le": 0, "shit": 1}.
    let out = [0x83 /* 3 (size) */, 0xa2, 0x65, 0x74 /* "et" */, 0xa5, 0x76, 0x6f, 0x69, 0x6c,
               0x61 /* "voila" */, 0xa2, 0x6c, 0x65 /* "le" */, 0x00 /* 0 */, 0xa4, 0x73,
               0x68, 0x69, 0x74 /* "shit" */, 0x01 /* 1 */];
    assert_eq!(out, buf);
}

#[test]
fn pass_enum() {
    let mut buf = Vec::new();

    #[derive(Serialize)]
    enum Enum {
        V1,
        V2,
    }

    Enum::V1.serialize(&mut Serializer::new(&mut buf)).unwrap();
    Enum::V2.serialize(&mut Serializer::new(&mut buf)).unwrap();

    // Expect: [0, []] [1, []].
    assert_eq!(vec![0x92, 0x00, 0x90, 0x92, 0x01, 0x90], buf);
}

#[test]
fn pass_tuple_enum_with_arg() {
    let mut buf = Vec::new();

    #[derive(Serialize)]
    enum Enum {
        V1,
        V2(u32),
    }

    Enum::V1.serialize(&mut Serializer::new(&mut buf)).unwrap();
    Enum::V2(42).serialize(&mut Serializer::new(&mut buf)).unwrap();

    // Expect: [0, []] [1, [42]].
    assert_eq!(vec![0x92, 0x00, 0x90, 0x92, 0x01, 0x91, 0x2a], buf);
}

#[test]
fn encode_struct_with_string_using_vec() {
    let mut buf = Vec::new();

    #[derive(Serialize)]
    struct Struct {
        f1: String,
    }

    let val = Struct {
        f1: "le message".into(),
    };
    val.serialize(&mut Serializer::new(&mut buf)).unwrap();

    // Expect: ["le message"].
    assert_eq!(vec![0x91, 0xaa, 0x6c, 0x65, 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65], buf);
}

#[test]
fn serialize_struct_variant() {
    let mut buf = Vec::new();

    #[derive(Serialize)]
    enum Enum {
        V1 {
            f1: u32,
        },
        V2 {
            f1: u32,
        },
    }

    Enum::V1 { f1: 42 }.serialize(&mut Serializer::new(&mut buf)).unwrap();
    Enum::V2 { f1: 43 }.serialize(&mut Serializer::new(&mut buf)).unwrap();

    // Expect: [0, [42]] [1, [42]].
    assert_eq!(vec![0x92, 0x00, 0x91, 0x2a, 0x92, 0x01, 0x91, 0x2b], buf);
}
