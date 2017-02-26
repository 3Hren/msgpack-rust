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

    fn mirror_f32(xs: f32) -> bool {
        let mut buf = Vec::new();
        encode::write_f32(&mut buf, xs).unwrap();

        xs == decode::read_f32(&mut &buf[..]).unwrap()
    }

    fn mirror_f64(xs: f64) -> bool {
        let mut buf = Vec::new();
        encode::write_f64(&mut buf, xs).unwrap();

        xs == decode::read_f64(&mut &buf[..]).unwrap()
    }
}
