use rmp::Marker;
use rmp::decode::MessageLen;
use rmp::decode::LenError;
use rmp::encode::*;

#[track_caller]
fn check_estimates(msg: &[u8], expected: &[i32]) {
    assert_eq!(msg.len()+1, expected.len(), "off by {}", msg.len() as isize + 1 - expected.len() as isize);

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
            assert!(predicted > partial_len.min(msg.len()-1), "{predicted} > {partial_len}/{}", msg.len());
            res
        })
        .collect::<Vec<_>>();
    assert_eq!(expected, predicted, "quadratic");
    assert_eq!(msg.len(), MessageLen::len_of(&msg).expect("complete message"));

    let mut incremental = MessageLen::with_limits(1024, 1<<16);
    let predicted = [&[][..]].into_iter().chain(msg.chunks(1)).map(|mut chunk| {
        let (res, _) = take_res(incremental.incremental_len(&mut chunk));
        res
    }).collect::<Vec<_>>();
    assert_eq!(expected, predicted, "incremental");

    for frag_len in [1, 2, 3, 5, 7] {
        let mut incremental = MessageLen::with_limits(1024, msg.len());
        let predicted = [&[][..]].into_iter().chain(msg.chunks(frag_len)).map(|mut chunk| {
            match incremental.incremental_len(&mut chunk) {
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
    write_u64(&mut out, 1<<33).unwrap();
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

    assert!(MessageLen::with_limits(4, 1<<16).incremental_len(&mut out.as_slice()).is_err());
    assert!(MessageLen::with_limits(14, 1<<16).incremental_len(&mut out.as_slice()).is_ok());
}
