use std::io;
use tokio::io::AsyncWriteExt;
use super::{ write_marker};
use crate::encode::ValueWriteError;
use crate::Marker;

/// Async Version of [`write_str_len`](crate::sync::encode::write_str_len)

pub async fn write_str_len<W: AsyncWriteExt +Unpin>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError<io::Error>> {
    if len < 32 {
        write_marker(wr, Marker::FixStr(len as u8)).await?;
        Ok(Marker::FixStr(len as u8))
    } else if len < 256 {
        write_marker(wr, Marker::Str8).await?;
        wr.write_u8(len as u8).await?;
        Ok(Marker::Str8)
    } else if len <= u16::MAX as u32 {
        write_marker(wr, Marker::Str16).await?;
        wr.write_u16(len as u16).await?;
        Ok(Marker::Str16)
    } else {
        write_marker(wr, Marker::Str32).await?;
        wr.write_u32(len).await?;
        Ok(Marker::Str32)
    }
}

/// Async Version of [`write_str`](crate::sync::encode::write_str)

pub async fn write_str<W: AsyncWriteExt +Unpin>(wr: &mut W, data: &str) -> Result<(), ValueWriteError<io::Error>> {
    write_str_len(wr, data.len() as u32).await?;
    wr.write_all(data.as_bytes()).await.map_err(ValueWriteError::InvalidDataWrite)
}
