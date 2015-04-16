mod null {
    use std::io::Cursor;

    use serialize::Decodable;

    use msgpack::Decoder;

    #[test]
    fn pass() {
        let buf = [0xc0];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!((), Decodable::decode(&mut decoder).ok().unwrap());
    }

    mod fail {
        use std::io::Cursor;
        use std::result;

        use serialize::Decodable;

        use msgpack::Decoder;
        use msgpack::core::decode::serialize::Error;

        type Result<T> = result::Result<T, Error>;

        #[test]
        fn from_reserved() {
            let buf = [0xc1];
            let cur = Cursor::new(&buf[..]);

            let mut decoder = Decoder::new(cur);

            let res: Result<()> = Decodable::decode(&mut decoder);
            assert_eq!(Error::TypeMismatch, res.err().unwrap());
        }
    } // mod fail
} // mod null

mod bool {
    use std::io::Cursor;

    use serialize::Decodable;

    use msgpack::Decoder;

    #[test]
    fn pass() {
        let buf = [0xc2, 0xc3];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(false, Decodable::decode(&mut decoder).ok().unwrap());
        assert_eq!(true,  Decodable::decode(&mut decoder).ok().unwrap());
    }

    mod fail {
        use std::io::Cursor;
        use std::result;

        use serialize::Decodable;

        use msgpack::Decoder;
        use msgpack::core::decode::serialize::Error;

        type Result<T> = result::Result<T, Error>;

        #[test]
        fn from_fixint() {
            let buf = [0x00];
            let cur = Cursor::new(&buf[..]);

            let mut decoder = Decoder::new(cur);

            let res: Result<bool> = Decodable::decode(&mut decoder);
            assert_eq!(Error::TypeMismatch, res.err().unwrap());
        }
    }
}

mod unspecified {
    use std::io::Cursor;
    use std::result;

    use serialize::Decodable;

    use msgpack::Decoder;
    use msgpack::core::decode::serialize::Error;

    type Result<T> = result::Result<T, Error>;

    #[test]
    fn pass_u64() {
        let buf = [0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(18446744073709551615u64, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn pass_u32() {
        let buf = [0xce, 0xff, 0xff, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(4294967295u32, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn fail_u32_from_u64() {
        let buf = [0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        let res: Result<u32> = Decodable::decode(&mut decoder);
        assert_eq!(Error::TypeMismatch, res.err().unwrap());
    }

    #[test]
    fn pass_u16() {
        let buf = [0xcd, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(65535u16, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn pass_u8() {
        let buf = [0xcc, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(255u8, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn pass_usize() {
        let buf = [0xcc, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(255us, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn pass_i64() {
        let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(9223372036854775807i64, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn pass_i32() {
        let buf = [0xd2, 0x7f, 0xff, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(2147483647i32, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn pass_i16() {
        let buf = [0xd1, 0x7f, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(32767i16, Decodable::decode(&mut decoder).ok().unwrap());
    }

    #[test]
    fn pass_i8() {
        let buf = [0xd0, 0x7f];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(127i8, Decodable::decode(&mut decoder).unwrap());
    }

    #[test]
    fn pass_isize() {
        let buf = [0xd0, 0x7f];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(127is, Decodable::decode(&mut decoder).unwrap());
    }

    #[test]
    fn pass_f32() {
        let buf = [0xca, 0x7f, 0x7f, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(3.4028234e38_f32, Decodable::decode(&mut decoder).unwrap());
    }

    #[test]
    fn pass_f64() {
        let buf = [0xcb, 0x40, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(42f64, Decodable::decode(&mut decoder).unwrap());
    }
}
