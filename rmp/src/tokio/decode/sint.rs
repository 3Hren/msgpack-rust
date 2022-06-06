use std::io;
use tokio::io::{AsyncReadExt};
use crate::decode::ValueReadError;
use crate::Marker;
use super::{read_marker};

/// Async Version of [`read_nfix`](crate::sync::decode::read_nfix)

pub async fn read_nfix<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<i8, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::FixNeg(val) => Ok(val),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_i8`](crate::sync::decode::read_i8)

pub async fn read_i8<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<i8, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::I8 => rd.read_i8().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_i16`](crate::sync::decode::read_i16)

pub async fn read_i16<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<i16, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::I16 => rd.read_i16().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_i32`](crate::sync::decode::read_i32)

pub async fn read_i32<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<i32, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::I32 => rd.read_i32().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_i64`](crate::sync::decode::read_i64)

pub async fn read_i64<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<i64, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::I64 => rd.read_i64().await.map_err(ValueReadError::from),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}
