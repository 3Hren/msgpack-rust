use std::io;
use tokio::io::AsyncWriteExt;
use crate::encode::ValueWriteError;
use super::{write_marker};

use crate::Marker;
use crate::tokio::encode::uint::{write_pfix, write_u16, write_u32, write_u64, write_u8};

/// Async Version of [`write_nfix`](crate::sync::encode::write_nfix)

#[inline]
#[track_caller]
pub async fn write_nfix<W: AsyncWriteExt + Unpin>(wr: &mut W, val: i8) -> Result<(), io::Error> {
    assert!(-32 <= val && val < 0);
    write_marker(wr, Marker::FixNeg(val)).await.map_err(|e| e.0)?;
    Ok(())
}

/// Async Version of [`write_i8`](crate::sync::encode::write_i8)

pub async fn write_i8<W: AsyncWriteExt + Unpin>(wr: &mut W, val: i8) -> Result<(), ValueWriteError<io::Error>> {
    write_marker(wr, Marker::I8).await?;
    wr.write_i8(val).await?;
    Ok(())
}
/// Async Version of [`write_i16`](crate::sync::encode::write_i16)

pub async fn write_i16<W: AsyncWriteExt + Unpin>(wr: &mut W, val: i16) -> Result<(), ValueWriteError<io::Error>> {
    write_marker(wr, Marker::I16).await?;
    wr.write_i16(val).await?;
    Ok(())
}

/// Async Version of [`write_i32`](crate::sync::encode::write_i32)

pub async fn write_i32<W: AsyncWriteExt + Unpin>(wr: &mut W, val: i32) -> Result<(), ValueWriteError<io::Error>> {
    write_marker(wr, Marker::I32).await?;
    wr.write_i32(val).await?;
    Ok(())
}

/// Async Version of [`write_i64`](crate::sync::encode::write_i64)

pub async fn write_i64<W: AsyncWriteExt + Unpin>(wr: &mut W, val: i64) -> Result<(), ValueWriteError<io::Error>> {
    write_marker(wr, Marker::I64).await?;
    wr.write_i64(val).await?;
    Ok(())
}
/// Async Version of [`write_sint`](crate::sync::encode::write_sint)

pub async fn write_sint<W: AsyncWriteExt + Unpin>(wr: &mut W, val: i64) -> Result<Marker, ValueWriteError<io::Error>> {
    match val {
        val if -32 <= val && val < 0 => {
            write_nfix(wr, val as i8).await
                .and(Ok(Marker::FixNeg(val as i8)))
                .map_err(ValueWriteError::InvalidMarkerWrite)
        }
        val if -128 <= val && val < -32 => write_i8(wr, val as i8).await.and(Ok(Marker::I8)),
        val if -32768 <= val && val < -128 => write_i16(wr, val as i16).await.and(Ok(Marker::I16)),
        val if -2147483648 <= val && val < -32768 => write_i32(wr, val as i32).await.and(Ok(Marker::I32)),
        val if val < -2147483648 => write_i64(wr, val).await.and(Ok(Marker::I64)),
        val if 0 <= val && val < 128 => {
            write_pfix(wr, val as u8).await
                .and(Ok(Marker::FixPos(val as u8)))
                .map_err(ValueWriteError::InvalidMarkerWrite)
        }
        val if val < 256 => write_u8(wr, val as u8).await.and(Ok(Marker::U8)),
        val if val < 65536 => write_u16(wr, val as u16).await.and(Ok(Marker::U16)),
        val if val < 4294967296 => write_u32(wr, val as u32).await.and(Ok(Marker::U32)),
        val => write_u64(wr, val as u64).await.and(Ok(Marker::U64)),
    }
}
