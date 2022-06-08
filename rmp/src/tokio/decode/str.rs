use core::str::{from_utf8};
use std::io;
use tokio::io::AsyncReadExt;

use super::{ read_marker, ValueReadError};
use crate::Marker;
use crate::decode::str::DecodeStringError;

/// Async Version of [`read_str_len`](crate::sync::decode::read_str_len)
#[inline]
pub async fn read_str_len<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<u32, ValueReadError<io::Error>> {
    Ok(read_str_len_with_nread(rd).await?.0)
}

async fn read_str_len_with_nread<R>(rd: &mut R) -> Result<(u32, usize), ValueReadError<io::Error>>
    where R: AsyncReadExt + Unpin
{
    match read_marker(rd).await? {
        Marker::FixStr(size) => Ok((size as u32, 1)),
        Marker::Str8 => Ok((rd.read_u8().await? as u32, 2)),
        Marker::Str16 => Ok((rd.read_u16().await? as u32, 3)),
        Marker::Str32 => Ok((rd.read_u32().await?, 5)),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_str`](crate::sync::decode::read_str)
pub async fn read_str<'r, R>(rd: &mut R, buf: &'r mut [u8]) -> Result<&'r str, DecodeStringError<'r, io::Error>>
where
    R: AsyncReadExt + Unpin,
{
    let len = read_str_len(rd).await?;
    let ulen = len as usize;

    if buf.len() < ulen {
        return Err(DecodeStringError::BufferSizeTooSmall(len));
    }

    read_str_data(rd, len, &mut buf[0..ulen]).await
}
/// Async Version of [`read_str_data`](crate::sync::decode::read_str_data)
pub async fn read_str_data<'r, R>(rd: &mut R,
                            len: u32,
                            buf: &'r mut [u8])
                            -> Result<&'r str, DecodeStringError<'r, io::Error>>
    where R: AsyncReadExt + Unpin
{
    debug_assert_eq!(len as usize, buf.len());

    // Trying to copy exact `len` bytes.
    match rd.read_exact(buf).await {
        Ok(_) => match from_utf8(buf) {
            Ok(decoded) => Ok(decoded),
            Err(err) => Err(DecodeStringError::InvalidUtf8(buf, err)),
        },
        Err(err) => Err(DecodeStringError::InvalidDataRead(err)),
    }
}


