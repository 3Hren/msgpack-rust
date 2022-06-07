use std::io;
use tokio::io::AsyncWriteExt;
use super::{ write_marker};
use crate::encode::ValueWriteError;
use crate::Marker;


pub async fn write_f32<W: AsyncWriteExt + Unpin>(wr: &mut W, val: f32) -> Result<(), ValueWriteError<io::Error>> {
    write_marker(wr, Marker::F32).await?;
    wr.write_f32(val).await?;
    Ok(())
}


pub async fn write_f64<W: AsyncWriteExt + Unpin>(wr: &mut W, val: f64) -> Result<(), ValueWriteError<io::Error>> {
    write_marker(wr, Marker::F64).await?;
    wr.write_f64(val).await?;
    Ok(())
}
