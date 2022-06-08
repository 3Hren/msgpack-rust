mod dec;
mod ext;
mod sint;
mod uint;

use std::io;
use num_traits::FromPrimitive;
use tokio::io::{AsyncRead, AsyncReadExt};
use crate::decode::{MarkerReadError, NumValueReadError, ValueReadError};
use crate::Marker;
pub use dec::{read_f32, read_f64};
pub use ext::{
    read_ext_meta, read_fixext1, read_fixext16, read_fixext2, read_fixext4, read_fixext8,
};
pub use sint::{read_i16, read_i32, read_i64, read_i8, read_nfix};
// While we re-export deprecated items, we don't want to trigger warnings while compiling this crate
pub use uint::{read_pfix, read_u16, read_u32, read_u64, read_u8};

/// Attempts to read a single byte from the given reader and to decode it as a MessagePack marker.
#[inline]
pub async fn read_marker<R: AsyncRead + Unpin>(rd: &mut R) -> Result<Marker, MarkerReadError<std::io::Error>> {
    Ok(Marker::from_u8(rd.read_u8().await?))
}

/// Async Version of [`read_marker`](crate::sync::decode::read_marker)
pub async fn read_nil<R: AsyncRead + Unpin>(rd: &mut R) -> Result<(), ValueReadError<std::io::Error>> {
    match read_marker(rd).await? {
        Marker::Null => Ok(()),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_bool`](crate::sync::decode::read_bool)
pub async fn read_bool<R: AsyncRead + Unpin>(rd: &mut R) -> Result<bool, ValueReadError<std::io::Error>> {
    match read_marker(rd).await? {
        Marker::True => Ok(true),
        Marker::False => Ok(false),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_int`](crate::sync::decode::read_int)
pub async fn read_int<T: FromPrimitive, R: AsyncRead + Unpin>(rd: &mut R) -> Result<T, NumValueReadError<std::io::Error>> {
    let val = match read_marker(rd).await? {
        Marker::FixPos(val) => T::from_u8(val),
        Marker::FixNeg(val) => T::from_i8(val),
        Marker::U8 => T::from_u8(rd.read_u8().await?),
        Marker::U16 => T::from_u16(rd.read_u16().await?),
        Marker::U32 => T::from_u32(rd.read_u32().await?),
        Marker::U64 => T::from_u64(rd.read_u64().await?),
        Marker::I8 => T::from_i8(rd.read_i8().await?),
        Marker::I16 => T::from_i16(rd.read_i16().await?),
        Marker::I32 => T::from_i32(rd.read_i32().await?),
        Marker::I64 => T::from_i64(rd.read_i64().await?),
        marker => return Err(NumValueReadError::TypeMismatch(marker)),
    };

    val.ok_or(NumValueReadError::OutOfRange)
}
/// Async Version of [`read_array_len`](crate::sync::decode::read_array_len)
pub async fn read_array_len<R>(rd: &mut R) -> Result<u32, ValueReadError<io::Error>>
    where
        R: AsyncRead + Unpin,
{
    match read_marker(rd).await? {
        Marker::FixArray(size) => Ok(size as u32),
        Marker::Array16 => Ok(rd.read_u16().await? as u32),
        Marker::Array32 => Ok(rd.read_u32().await?),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_map_len`](crate::sync::decode::read_map_len)
pub async fn read_map_len<R: AsyncRead + Unpin>(rd: &mut R) -> Result<u32, ValueReadError<io::Error>> {
    let marker = read_marker(rd).await?;
    marker_to_len(rd, marker).await
}
/// Async Version of [`marker_to_len`](crate::sync::decode::marker_to_len)
pub async fn marker_to_len<R: AsyncRead + Unpin>(rd: &mut R, marker: Marker) -> Result<u32, ValueReadError<io::Error>> {
    match marker {
        Marker::FixMap(size) => Ok(size as u32),
        Marker::Map16 => Ok(rd.read_u16().await? as u32),
        Marker::Map32 => Ok(rd.read_u32().await?),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}

/// Async Version of [`read_bin_len`](crate::sync::decode::read_bin_len)
pub async fn read_bin_len<R: AsyncRead + Unpin>(rd: &mut R) -> Result<u32, ValueReadError<io::Error>> {
    match read_marker(rd).await? {
        Marker::Bin8 => Ok(rd.read_u8().await? as u32),
        Marker::Bin16 => Ok(rd.read_u16().await? as u32),
        Marker::Bin32 => Ok(rd.read_u32().await?),
        marker => Err(ValueReadError::TypeMismatch(marker)),
    }
}