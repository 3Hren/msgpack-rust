extern crate rmpv;
#[macro_use]
extern crate quickcheck;

use rmpv::Value;
use rmpv::decode::read_value;
use rmpv::encode::write_value;

quickcheck! {
    fn mirror_uint(xs: u64) -> bool {
        let mut buf = Vec::new();
        write_value(&mut buf, &Value::U64(xs)).unwrap();

        Value::U64(xs) == read_value(&mut &buf[..]).unwrap()
    }
}
