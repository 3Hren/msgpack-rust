#[macro_use]
extern crate quickcheck;

use rmpv::Value;
use rmpv::decode::read_value;
use rmpv::encode::write_value;

fn mirror_test<T: Clone>(xs: T) -> bool
    where Value: From<T>
{
    let mut buf = Vec::new();
    write_value(&mut buf, &Value::from(xs.clone())).unwrap();

    Value::from(xs) == read_value(&mut &buf[..]).unwrap()
}

quickcheck! {
    fn mirror_uint(xs: u64) -> bool {
        mirror_test(xs)
    }

    fn mirror_sint(xs: i64) -> bool {
        mirror_test(xs)
    }

    fn mirror_f32(xs: f32) -> bool {
        mirror_test(xs)
    }

    fn mirror_f64(xs: f64) -> bool {
        mirror_test(xs)
    }

    fn mirror_str(xs: String) -> bool {
        mirror_test(xs)
    }
}
