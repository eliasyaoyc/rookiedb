use std::{io::SeekFrom, ptr::NonNull};

use async_fs::File;
use futures_lite::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::error::Result;

pub(crate) struct PageFile(pub(crate) File);

impl PageFile {
    #[inline]
    pub(crate) async fn read(&mut self, ouput: &mut [u8]) -> Result<()> {
        self.0.read(ouput).await?;
        self.0.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    #[inline]
    pub(crate) async fn read_from(&mut self, offset: u64, ouput: &mut [u8]) -> Result<()> {
        self.0.seek(SeekFrom::Start(offset)).await?;
        self.0.read(ouput).await?;
        self.0.seek(SeekFrom::Start(0)).await?;
        Ok(())
    }

    #[inline]
    pub(crate) async fn write_to(&mut self, offset: u64, buf: &[u8]) -> Result<()> {
        self.0.seek(SeekFrom::Start(offset)).await?;
        self.0.write(buf).await?;
        Ok(())
    }

    #[inline]
    pub(crate) async fn write_to_f<F>(&mut self, offset: u64, f: F) -> Result<()>
    where
        F: FnOnce() -> Vec<u8>,
    {
        self.0.seek(SeekFrom::Start(offset)).await?;
        self.0.write(&f()).await?;
        Ok(())
    }

    #[inline]
    pub(crate) async fn write_f<F>(&mut self, f: F) -> Result<()>
    where
        F: FnOnce() -> Vec<u8>,
    {
        self.0.write(&f()).await?;
        Ok(())
    }
}

/// Page represents a page loaded in memory (as opposed to the buffer frame it's
/// in). Wraps around buffer manager frames, and requests the page be loaded
/// into memory as necessary.
///
/// Header pages are layed out as follows:
///  - first byte: 0x1 to indicate valid allocated page
///  - next 4 bytes: page group id
///  - next 8 bytes: page number of next header page, (or -1) if no next header
///    page.
///  - next 10 bytes: page number of data page (or -1), followed by 2 bytes of
///    amount of free space
///  - repeat 10 byte entries
///
/// Data pages contain a small header containing:
///  - 4-byte page group id
///  - 4-byte index of which header page manages it
///  - 2-byte offset indicating which slot in the header page its data page
///    entry resides
///
/// This header is used to quickly locate and update the header page when the
/// amount of free space on the data page changes, as well as ensure that we do
/// not modify pages in other page groups by accident.
///
/// The page group id is a randomly generated 32-bit integer used to help
/// detect bugs (where we attempt to write to a page that is not managed by the
/// page group).
pub struct HeaderPage {
    data: Vec<u8>,
    data_page_nums: usize,
    header_offset: usize,

    next: Option<NonNull<HeaderPage>>,
}

impl HeaderPage {
    pub fn new(page_num: usize, header_offset: usize, first: bool) -> HeaderPage {
        HeaderPage {
            data: Vec::new(),
            data_page_nums: 0,
            header_offset,
            next: None,
        }
    }
}

pub struct DataPage {
    data: Vec<u8>,
}

#[cfg(test)]
mod tests {}
