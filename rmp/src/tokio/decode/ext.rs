use std::io;
use tokio::io::AsyncReadExt;
use crate::decode::{ExtMeta, ValueReadError};
use crate::Marker;
use super::read_marker;

/// Async Version of [`read_fixext1`](crate::sync::decode::read_fixext1)
pub async fn read_fixext1<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<(i8, u8), ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::FixExt1 => {
            let ty = rd.read_i8().await?;
            let data = rd.read_u8().await?;
            Ok((ty, data))
        }
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_fixext2`](crate::sync::decode::read_fixext2)

pub async fn read_fixext2<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<(i8, [u8; 2]), ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::FixExt2 => {
            let mut buf = [0; 2];
            let id = rd.read_i8().await?;
            rd.read_exact(&mut buf).await.map_err(ValueReadError::InvalidDataRead)?;
            Ok((id, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_fixext4`](crate::sync::decode::read_fixext4)

pub async fn read_fixext4<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<(i8, [u8; 4]), ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::FixExt4 => {
            let mut buf = [0; 4];
            let id = rd.read_i8().await?;
            rd.read_exact(&mut buf).await.map_err(ValueReadError::InvalidDataRead)?;
            Ok((id, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_fixext8`](crate::sync::decode::read_fixext8)

pub async fn read_fixext8<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<(i8, [u8; 8]), ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::FixExt8 => {
            let mut buf = [0; 8];
            let id = rd.read_i8().await?;
            rd.read_exact(&mut buf).await.map_err(ValueReadError::InvalidDataRead)?;
            Ok((id, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_fixext16`](crate::sync::decode::read_fixext16)

pub async fn read_fixext16<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<(i8, [u8; 16]), ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::FixExt16 => {
            let mut buf = [0; 16];
            let id = rd.read_i8().await?;
            rd.read_exact(&mut buf).await.map_err(ValueReadError::InvalidDataRead)?;
            Ok((id, buf))
        }
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}



/// Async Version of [`read_ext_meta`](crate::sync::decode::read_ext_meta)

pub async fn read_ext_meta<R: AsyncReadExt + Unpin>(rd: &mut R) -> Result<ExtMeta, ValueReadError<io::Error>> {
    let size = match read_marker(rd).await? {
        Marker::FixExt1 => 1,
        Marker::FixExt2 => 2,
        Marker::FixExt4 => 4,
        Marker::FixExt8 => 8,
        Marker::FixExt16 => 16,
        Marker::Ext8 => rd.read_u8().await? as u32,
        Marker::Ext16 => rd.read_u16().await? as u32,
        Marker::Ext32 => rd.read_u32().await?,
        marker => return Err(ValueReadError::TypeMismatch(marker)),
    };

    let ty = rd.read_i8().await?;
    let meta = ExtMeta { typeid: ty, size };

    Ok(meta)
}
