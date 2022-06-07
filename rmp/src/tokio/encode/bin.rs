use std::io;
use tokio::io::AsyncWriteExt;
use crate::encode::ValueWriteError;
use crate::Marker;
use super::write_marker;

/// Async Version of [`write_bin_len`](crate::sync::encode::write_bin_len)
pub async fn write_bin_len<W: AsyncWriteExt + Unpin>(wr: &mut W, len: u32) -> Result<Marker, ValueWriteError<io::Error>> {
    if len < 256 {
        write_marker(&mut *wr, Marker::Bin8).await?;
        wr.write_u8(len as u8).await?;
        Ok(Marker::Bin8)
    } else if len <= u16::MAX as u32 {
        write_marker(&mut *wr, Marker::Bin16).await?;
        wr.write_u16(len as u16).await?;
        Ok(Marker::Bin16)
    } else {
        write_marker(&mut *wr, Marker::Bin32).await?;
        wr.write_u32(len).await?;
        Ok(Marker::Bin32)
    }
}

/// Async Version of [`write_bin`](crate::sync::encode::write_bin)
pub async fn write_bin<W: AsyncWriteExt + Unpin>(wr: &mut W, data: &[u8]) -> Result<(), ValueWriteError<io::Error>> {
    write_bin_len(wr, data.len() as u32).await?;
    wr.write_all(data).await
        .map_err(ValueWriteError::InvalidDataWrite)
}
