use std::io;
use tokio::io::AsyncWriteExt;
use crate::encode::{ValueWriteError};
use crate::Marker;
use super::write_marker;

/// Async Version of [`write_pfix`](crate::sync::encode::write_pfix)
#[inline]
pub async fn write_pfix<W: AsyncWriteExt + Unpin>(wr: &mut W, val: u8) -> Result<(), io::Error>{
    assert!(val < 128);
    write_marker(wr, Marker::FixPos(val)).await.map_err(|e| e.0)?;
    Ok(())
}

/// Async Version of [`write_u8`](crate::sync::encode::write_u8)
pub async fn write_u8<W: AsyncWriteExt + Unpin>(wr: &mut W, val: u8) -> Result<(), ValueWriteError<io::Error>>{
    write_marker(wr, Marker::U8).await?;
    wr.write_u8(val).await?;
    Ok(())
}

/// Async Version of [`write_u16`](crate::sync::encode::write_u16)
pub async fn write_u16<W: AsyncWriteExt + Unpin>(wr: &mut W, val: u16) -> Result<(), ValueWriteError<io::Error>>{
    write_marker(wr, Marker::U16).await?;
    wr.write_u16(val).await?;
    Ok(())
}

/// Async Version of [`write_u32`](crate::sync::encode::write_u32)
pub async fn write_u32<W: AsyncWriteExt + Unpin>(wr: &mut W, val: u32) -> Result<(), ValueWriteError<io::Error>>{
    write_marker(wr, Marker::U32).await?;
    wr.write_u32(val).await?;
    Ok(())
}

/// Async Version of [`write_u64`](crate::sync::encode::write_u64)
pub async fn write_u64<W: AsyncWriteExt + Unpin>(wr: &mut W, val: u64) -> Result<(), ValueWriteError<io::Error>>{
    write_marker(wr, Marker::U64).await?;
    wr.write_u64(val).await?;
    Ok(())
}

/// Async Version of [`write_uint`](crate::sync::encode::write_uint)
pub async fn write_uint<W: AsyncWriteExt + Unpin>(wr: &mut W, val: u64) -> Result<Marker, ValueWriteError<io::Error>>{
    if val < 128 {
        write_pfix(wr, val as u8).await
            .and(Ok(Marker::FixPos(val as u8)))
            .map_err(ValueWriteError::InvalidMarkerWrite)
    } else if val < 256 {
        write_u8(wr, val as u8).await.and(Ok(Marker::U8))
    } else if val < 65536 {
        write_u16(wr, val as u16).await.and(Ok(Marker::U16))
    } else if val < 4294967296 {
        write_u32(wr, val as u32).await.and(Ok(Marker::U32))
    } else {
        write_u64(wr, val).await.and(Ok(Marker::U64))
    }
}
