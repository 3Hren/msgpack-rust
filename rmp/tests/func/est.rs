#![cfg(feature = "std")]

use rmp::decode::{LenError, MessageLen};
use rmp::encode::*;
use rmp::Marker;

#[track_caller]
fn check_estimates(msg: &[u8], expected: &[i32]) {
    assert_eq!(msg.len() + 1, expected.len(), "off by {}", msg.len() as isize + 1 - expected.len() as isize);

    fn take_res(r: Result<usize, LenError>) -> (i32, usize) {
        match r {
            Err(r) => (r.len() as i32, r.len()),
            Ok(r) => (-(r as i32), r),
        }
    }

    let predicted = (0..=msg.len())
        .map(|partial_len| {
            let partial_msg = &msg[..partial_len];
            let (res, predicted) = take_res(MessageLen::len_of(partial_msg));
            assert!(predicted > partial_len.min(msg.len() - 1), "{predicted} > {partial_len}/{}", msg.len());
            res
        })
        .collect::<Vec<_>>();
    assert_eq!(expected, predicted, "quadratic");
    assert_eq!(msg.len(), MessageLen::len_of(msg).expect("complete message"));

    let mut incremental = MessageLen::with_limits(1024, 1<<16);
    let predicted = [&[][..]].into_iter().chain(msg.chunks(1)).map(|chunk| {
        let (res, _) = take_res(incremental.incremental_len(chunk));
        res
    }).collect::<Vec<_>>();
    assert_eq!(expected, predicted, "incremental");

    for frag_len in [1, 2, 3, 5, 7] {
        let mut incremental = MessageLen::with_limits(1024, msg.len());
        let predicted = [&[][..]].into_iter().chain(msg.chunks(frag_len)).map(|chunk| {
            match incremental.incremental_len(chunk) {
                Err(r) => r.len(),
                Ok(r) => r,
            }
        }).max().unwrap_or(usize::MAX);
        assert_eq!(msg.len(), predicted, "incremental {frag_len}");
    }
}

#[test]
fn array() {
    assert_eq!(1, MessageLen::len_of(&[]).unwrap_err().len());

    let mut out = [0u8; 1];
    write_bool(&mut out.as_mut_slice(), true).unwrap();
    assert_eq!(1, MessageLen::len_of(&out).unwrap());

    let mut out = Vec::new();
    write_array_len(&mut out, 4).unwrap();
    write_u16(&mut out, 333).unwrap();
    write_bool(&mut out, true).unwrap();
    write_u64(&mut out, 1 << 33).unwrap();
    write_bin_len(&mut out, 5).unwrap();
    out.extend(b"hello");

    check_estimates(&out, &[1, 5, 5, 5, 7, 7, 14, 14, 14, 14, 14, 14, 14, 14, 15, 16, 21, 21, 21, 21, 21, -21]);
}

#[test]
fn map() {
    let mut out = Vec::new();
    write_map_len(&mut out, 3).unwrap();
        write_u16(&mut out, 333).unwrap();
        write_bool(&mut out, true).unwrap();

        write_str_len(&mut out, 5).unwrap();
        out.extend(b"hello");
        write_nil(&mut out).unwrap();

        write_f64(&mut out, 1.23).unwrap();
        write_map_len(&mut out, 2).unwrap();
            write_nil(&mut out).unwrap();
            out.push(Marker::Array32.to_u8());
                out.extend_from_slice(&1u32.to_be_bytes());
                write_uint8(&mut out, 3).unwrap();
            write_nil(&mut out).unwrap();
            write_u32(&mut out, 1).unwrap();


    check_estimates(&out, &[1, 7, 7, 7, 9, 9, 11, 11, 11, 11, 11, 14, 14, 21, 21, 21, 21, 21, 21, 21, 21, 22, 26, 26, 28, 28, 28, 28, 29, 31, 31, 35, 35, 35, 35, -35]);
}

#[test]
fn nested() {
    let mut out = Vec::new();
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 1).unwrap();
    out.push(Marker::Array32.to_u8());
    out.extend_from_slice(&1u32.to_be_bytes());
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 2).unwrap();
    write_array_len(&mut out, 1).unwrap();
    out.push(Marker::Array16.to_u8());
    out.extend_from_slice(&1u16.to_be_bytes());
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 1).unwrap();
    write_array_len(&mut out, 1).unwrap();
    write_nil(&mut out).unwrap();
    write_nil(&mut out).unwrap();

    check_estimates(&out, &[1, 2, 3, 4, 8, 8, 8, 8, 9, 10, 11, 12, 14, 14, 16, 16, 17, 18, 19, 20, 21, 22, -22]);

    assert!(MessageLen::with_limits(4, 1 << 16).incremental_len(out.as_slice()).is_err());
    assert!(MessageLen::with_limits(14, 1 << 16).incremental_len(out.as_slice()).is_ok());
}

#[test]
fn extensions() {
    let mut out = Vec::with_capacity(263);

    let mut expected = Vec::with_capacity(264);
    expected.push(1);

    for len in (0i32..=17).chain([255, 256, 257, 65535]) {
        const TOO_BIG_FOR_U8: i32 = u8::MAX as i32 + 1;
        const TOO_BIG_FOR_U16: i32 = u16::MAX as i32 + 1;

        let length_bytes = match len {
            1 | 2 | 4 | 8 | 16 => 0,
            ..TOO_BIG_FOR_U8 => 1,
            TOO_BIG_FOR_U8..TOO_BIG_FOR_U16 => 2,
            _ => 4,
        };

        let len_with_prefix = |len| 1 + length_bytes + 1 + len;
        let msg_len = len_with_prefix(len);

        expected.truncate(1);
        // while the length is being read, the type byte is already known to follow it
        expected.resize(1 + length_bytes as usize, 2 + length_bytes);
        expected.resize(msg_len as usize, msg_len);
        expected.push(-msg_len);

        out.clear();
        write_ext_meta(&mut out, len as u32, 0x67).unwrap();
        out.resize(out.len() + len as usize, 0xab);

        check_estimates(&out, &expected);
    }
}

#[test]
fn ext32() {
    // Non-canonical but valid: ext32 marker with a small length, so the estimates can be
    // checked exhaustively for every prefix.
    let mut out = vec![Marker::Ext32.to_u8()];
    out.extend_from_slice(&3u32.to_be_bytes());
    out.push(7);
    out.extend_from_slice(&[0xEE; 3]);
    // marker + 4 len bytes + type byte + data
    check_estimates(&out, &[1, 6, 6, 6, 6, 9, 9, 9, 9, -9]);

    // Canonical ext32 as produced by the encoder.
    let len = 70_000u32;
    let mut out = Vec::new();
    assert_eq!(Marker::Ext32, write_ext_meta(&mut out, len, 7).unwrap());
    out.extend(std::iter::repeat_n(0xEE, len as usize));
    assert_eq!(out.len(), MessageLen::len_of(&out).unwrap());
    assert_eq!(out.len(), MessageLen::len_of(&out[..out.len() - 1]).unwrap_err().len());
}

#[test]
fn ext_in_array() {
    let mut out = Vec::new();
    write_array_len(&mut out, 3).unwrap();
    write_ext_meta(&mut out, 1, 7).unwrap(); // fixext1
    out.push(0xEE);
    write_ext_meta(&mut out, 3, 7).unwrap(); // ext8
    out.extend_from_slice(&[0xEE; 3]);
    write_nil(&mut out).unwrap();
    assert_eq!(11, out.len());

    check_estimates(&out, &[1, 4, 4, 4, 6, 7, 10, 10, 10, 10, 11, -11]);
}

#[test]
fn ext_in_map() {
    let mut out = Vec::new();
    write_map_len(&mut out, 1).unwrap();
    write_ext_meta(&mut out, 8, 1).unwrap(); // fixext8 key
    out.extend_from_slice(&[0xEE; 8]);
    write_ext_meta(&mut out, 300, 2).unwrap(); // ext16 value
    out.extend_from_slice(&[0xEE; 300]);
    // 1 + (2 + 8) + (4 + 300)
    assert_eq!(315, out.len());

    let mut expected = vec![1, 3];
    expected.extend(std::iter::repeat_n(11, 9)); // key: type byte + 8 data bytes
    expected.extend([12, 15, 15]); // value: marker, then 2 length bytes + type byte
    expected.resize(315, 315);
    expected.push(-315);
    check_estimates(&out, &expected);
}

#[test]
fn limit_exceeded_is_parse_error() {
    // Nesting limit, all data available in the first call.
    let mut out = Vec::new();
    for _ in 0..5 {
        write_array_len(&mut out, 1).unwrap();
    }
    write_nil(&mut out).unwrap();
    let mut est = MessageLen::with_limits(4, 1 << 16);
    assert!(matches!(est.incremental_len(&out), Err(LenError::ParseError)));
    // The error is sticky.
    assert!(matches!(est.incremental_len(&out), Err(LenError::ParseError)));
    assert_eq!(6, MessageLen::with_limits(5, 1 << 16).incremental_len(&out).unwrap());

    // Nesting limit hit while resuming from a truncated message.
    let mut est = MessageLen::with_limits(2, 1 << 16);
    assert_eq!(2, est.incremental_len(&[0x91]).unwrap_err().len());
    assert!(matches!(est.incremental_len(&[0x91, 0x91, 0xc0]), Err(LenError::ParseError)));

    // `len_of` has a fixed nesting limit of 1024.
    let mut out = Vec::new();
    for _ in 0..1025 {
        write_array_len(&mut out, 1).unwrap();
    }
    write_nil(&mut out).unwrap();
    assert!(matches!(MessageLen::len_of(&out), Err(LenError::ParseError)));

    // Length limit is exclusive, for strings...
    let mut out = Vec::new();
    write_str_len(&mut out, 40).unwrap(); // str8
    out.extend_from_slice(&[b'x'; 40]);
    assert!(matches!(MessageLen::with_limits(1024, 40).incremental_len(&out), Err(LenError::ParseError)));
    assert_eq!(42, MessageLen::with_limits(1024, 41).incremental_len(&out).unwrap());

    // ...arrays (counted in items)...
    let mut out = Vec::new();
    write_array_len(&mut out, 40).unwrap(); // array16
    out.extend(std::iter::repeat_n(0xc0, 40));
    assert!(matches!(MessageLen::with_limits(1024, 40).incremental_len(&out), Err(LenError::ParseError)));
    assert_eq!(43, MessageLen::with_limits(1024, 41).incremental_len(&out).unwrap());

    // ...maps (counted in keys + values)...
    let mut out = Vec::new();
    write_map_len(&mut out, 20).unwrap(); // map16
    out.extend(std::iter::repeat_n(0xc0, 40));
    assert!(matches!(MessageLen::with_limits(1024, 40).incremental_len(&out), Err(LenError::ParseError)));
    assert_eq!(43, MessageLen::with_limits(1024, 41).incremental_len(&out).unwrap());

    // ...and ext payloads.
    let mut out = Vec::new();
    write_ext_meta(&mut out, 40, 7).unwrap(); // ext8
    out.extend_from_slice(&[0xEE; 40]);
    assert!(matches!(MessageLen::with_limits(1024, 40).incremental_len(&out), Err(LenError::ParseError)));
    assert_eq!(43, MessageLen::with_limits(1024, 41).incremental_len(&out).unwrap());
}

#[test]
fn reserved_marker_is_parse_error() {
    assert!(matches!(MessageLen::len_of(&[0xc1]), Err(LenError::ParseError)));
    assert!(matches!(MessageLen::len_of(&[0x92, 0xc0, 0xc1]), Err(LenError::ParseError)));
    // Truncation before the reserved byte is still just truncation.
    assert_eq!(3, MessageLen::len_of(&[0x92, 0xc0]).unwrap_err().len());

    // The error is sticky.
    let mut est = MessageLen::new();
    assert_eq!(3, est.incremental_len(&[0x92, 0xc0]).unwrap_err().len());
    assert!(matches!(est.incremental_len(&[0xc1]), Err(LenError::ParseError)));
    assert!(matches!(est.incremental_len(&[0xc0]), Err(LenError::ParseError)));
}
