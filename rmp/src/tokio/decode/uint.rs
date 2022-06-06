use std::io;
use tokio::io::AsyncReadExt;
use crate::Marker;
use super::{read_marker, ValueReadError};

/// Async Version of [`read_pfix`](crate::sync::decode::read_pfix)

pub async fn read_pfix<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<u8, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::FixPos(val) => Ok(val),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_u8`](crate::sync::decode::read_u8)

pub async fn read_u8<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<u8, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::U8 => rd.read_u8().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_u16`](crate::sync::decode::read_u16)

pub async fn read_u16<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<u16, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::U16 => rd.read_u16().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_u32`](crate::sync::decode::read_u32)

pub async fn read_u32<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<u32, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::U32 => rd.read_u32().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_u64`](crate::sync::decode::read_u64)

pub async fn read_u64<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<u64, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::U64 => rd.read_u64().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}
