use msgpack::{decode, encode};

quickcheck! {
    fn mirror_uint(xs: u64) -> bool {
        let mut buf = Vec::new();
        encode::write_uint(&mut buf, xs).unwrap();

        xs == decode::read_int(&mut &buf[..]).unwrap()
    }

    fn mirror_sint(xs: i64) -> bool {
        let mut buf = Vec::new();
        encode::write_sint(&mut buf, xs).unwrap();

        xs == decode::read_int(&mut &buf[..]).unwrap()
    }
}
