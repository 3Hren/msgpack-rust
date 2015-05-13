extern crate test;

use super::decode::*;
use self::test::Bencher;

#[bench]
fn from_i64_read_i64_loosely(b: &mut Bencher) {
    let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

    b.iter(|| {
        let res = read_i64_loosely(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}

#[bench]
fn from_i64_read_integer(b: &mut Bencher) {
    let buf = [0xd3, 0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];

    b.iter(|| {
        let res = read_integer(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}

#[bench]
fn from_i8_read_i8(b: &mut Bencher) {
    let buf = [0xd0, 0xff];

    b.iter(|| {
        let res = read_i8(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}

#[bench]
fn from_u8_read_u64_loosely(b: &mut Bencher) {
    let buf = [0xcc, 0xff];

    b.iter(|| {
        let res = read_u64_loosely(&mut &buf[..]).unwrap();
        test::black_box(res);
    });
}
