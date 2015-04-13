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

    use serialize::Decodable;

    use msgpack::Decoder;

    #[test]
    fn pass_u64() {
        let buf = [0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let cur = Cursor::new(&buf[..]);

        let mut decoder = Decoder::new(cur);

        assert_eq!(18446744073709551615u64, Decodable::decode(&mut decoder).ok().unwrap());
    }
}
