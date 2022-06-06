use std::io;
use tokio::io::{ AsyncReadExt};
use crate::decode::ValueReadError;
use crate::Marker;
use super::read_marker;

/// Async Version of [`read_f32`](crate::sync::decode::read_f32)
pub async fn read_f32<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<f32, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::F32 => Ok(rd.read_f32().await?),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_f64`](crate::sync::decode::read_f64)
pub async fn read_f64<R:  AsyncReadExt + Unpin>(rd: &mut R) -> Result<f64, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::F64 => Ok(rd.read_f64().await?),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}