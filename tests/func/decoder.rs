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
            let buf = [0xc3];
            let cur = Cursor::new(&buf[..]);

            let mut decoder = Decoder::new(cur);

            let res: Result<()> = Decodable::decode(&mut decoder);
            assert_eq!(Error::TypeMismatch, res.err().unwrap());
        }
    } // mod fail
} // mod null
