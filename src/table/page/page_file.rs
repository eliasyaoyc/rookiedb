use std::io::SeekFrom;

use async_fs::File;
use futures_lite::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::error::Result;

pub struct PageFile(pub File);

impl PageFile {
    #[inline]
    pub async fn read(&mut self, ouput: &mut [u8]) -> Result<()> {
        self.0.read(ouput).await?;
        self.0.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    #[inline]
    pub async fn read_from(&mut self, offset: u64, ouput: &mut [u8]) -> Result<()> {
        self.0.seek(SeekFrom::Start(offset)).await?;
        self.0.read(ouput).await?;
        self.0.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    #[inline]
    pub async fn write_to(&mut self, offset: u64, buf: &[u8]) -> Result<()> {
        self.0.seek(SeekFrom::Start(offset)).await?;
        self.0.write(buf).await?;
        Ok(())
    }

    #[inline]
    pub async fn write_to_f<F>(&mut self, offset: u64, f: F) -> Result<()>
    where
        F: FnOnce() -> Vec<u8>,
    {
        self.0.seek(SeekFrom::Start(offset)).await?;
        self.0.write(&f()).await?;
        Ok(())
    }

    #[inline]
    pub async fn write_f<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce() -> Vec<u8>,
    {
        self.0.write(&f()).await?;
        Ok(())
    }
}
