use std::io;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use crate::encode::{MarkerWriteError, ValueWriteError};
use crate::Marker;

async fn write_marker<W: AsyncWriteExt + Unpin>(wr: &mut W, marker: Marker) -> Result<(), MarkerWriteError<io::Error>> {
    wr.write_u8(marker.to_u8()).await.map_err(MarkerWriteError)
}

/// Async version of [`write_nil`](crate::sync::encode::write_nil).
#[inline]
pub async fn write_nil<W: AsyncWriteExt + Unpin>(wr: &mut W) -> Result<(), io::Error> {
    write_marker(wr, Marker::Null).await.map_err(|e| e.0)
}

/// Async version of [`write_bool`](crate::sync::encode::write_bool).
#[inline]
pub async fn write_bool<W: AsyncWriteExt + Unpin>(wr: &mut W, val: bool) -> Result<(), io::Error> {
    let marker = if val { Marker::True } else { Marker::False };

    write_marker(wr, marker).await.map_err(|e| e.0)
}

/// Async version of [`write_array_len`](crate::sync::encode::write_array_len).

pub async fn write_array_len<W: AsyncWriteExt + Unpin>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError<io::Error>> {
    let marker = if len < 16 {
        write_marker(wr, Marker::FixArray(len as u8)).await?;
        Marker::FixArray(len as u8)
    } else if len <= u16::MAX as u32 {
        write_marker(wr, Marker::Array16).await?;
        wr.write_u16(len as u16).await?;
        Marker::Array16
    } else {
        write_marker(wr, Marker::Array32).await?;
        wr.write_u32(len).await?;
        Marker::Array32
    };

    Ok(marker)
}

pub  async fn write_map_len<W: AsyncWriteExt + Unpin>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError<io::Error>> {
    let marker = if len < 16 {
        write_marker(wr, Marker::FixMap(len as u8)).await?;
        Marker::FixMap(len as u8)
    } else if len <= u16::MAX as u32 {
        write_marker(wr, Marker::Map16).await?;
        wr.write_u16(len as u16).await?;
        Marker::Map16
    } else {
        write_marker(wr, Marker::Map32).await?;
        wr.write_u32(len).await?;
        Marker::Map32
    };

    Ok(marker)
}

pub async fn write_ext_meta<W: AsyncWriteExt + Unpin>(wr: &mut W, len: u32, ty: i8) -> Result<Marker, ValueWriteError<io::Error>> {
    let marker = match len {
        1 => {
            write_marker(wr, Marker::FixExt1).await?;
            Marker::FixExt1
        }
        2 => {
            write_marker(wr, Marker::FixExt2).await?;
            Marker::FixExt2
        }
        4 => {
            write_marker(wr, Marker::FixExt4).await?;
            Marker::FixExt4
        }
        8 => {
            write_marker(wr, Marker::FixExt8).await?;
            Marker::FixExt8
        }
        16 => {
            write_marker(wr, Marker::FixExt16).await?;
            Marker::FixExt16
        }
        len if len < 256 => {
            write_marker(wr, Marker::Ext8).await?;
            wr.write_u8(len as u8).await?;
            Marker::Ext8
        }
        len if len < 65536 => {
            write_marker(wr, Marker::Ext16).await?;
            wr.write_u16(len as u16).await?;
            Marker::Ext16
        }
        len => {
            write_marker(wr, Marker::Ext32).await?;
            wr.write_u32(len).await?;
            Marker::Ext32
        }
    };

    wr.write_i8(ty).await?;

    Ok(marker)
}
