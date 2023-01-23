use std::{
    alloc::realloc,
    collections::HashMap,
    io::{ErrorKind, SeekFrom},
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

use async_fs::File;
use bytes::BufMut;
use futures_lite::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use parking_lot::Mutex;

use super::{
    cache::lru::Lru,
    page::{HeaderPage, PageFile},
};
use crate::{
    error::{Error, Result},
    utils::fs,
};

/// The default size of the page is 4KB.
const DEFAULT_PAGE_SIZE: usize = 4 * 1024;

const MAX_HEADER_PAGES: usize = DEFAULT_PAGE_SIZE / 2;

const DATA_PAGES_PER_HEADER: usize = DEFAULT_PAGE_SIZE * 8;

/// The size of header in header pages.
/// 1-byte  : check page is valid.
/// 4-bytes : page group id.
/// 8-bytes : next header page offset.
const HEADER_HEADER_SIZE: usize = 13;

/// The size of header in data pages.
const DATA_HEADER_SIZE: usize = 10;

/// The size of data entry.
/// 8-bytes : data page offset.
/// 2-bytes : free space in current data page.
const DATA_ENTRY_SIZE: usize = 10;

/// Reserve 36 bytes on each page for bookkeeping for recovery
/// (used to store the pageLSN, and to ensure that a redo-only/undo-only log
/// record can fit on one page).
const RESERVED_SIZE: usize = 36;

pub struct PartitionHandle {
    part_id: usize,

    page_file: PageFile,

    master_page: Vec<u16>,

    header_page: Vec<Vec<u8>>,
}

impl PartitionHandle {
    async fn create(part_id: usize) -> Result<Self> {
        // Open the os file and loads master and header pages.
        let mut page_file = match fs::open("").await {
            Ok(file) => PageFile(file),
            Err(error) => {
                if ErrorKind::NotFound == error.kind() {
                    // File not exist and create new file.
                    PageFile(fs::create_file("").await?)
                } else {
                    return Err(Error::IO(error));
                }
            }
        };

        let mut master_page = Vec::with_capacity(MAX_HEADER_PAGES);
        let mut header_page = Vec::with_capacity(DATA_PAGES_PER_HEADER);

        let page_len = page_file.0.metadata().await?.len();
        if page_len == 0 {
            // New file, write empty master page.
            Self::write_master_page(&mut page_file.0, &master_page).await?;
        } else {
            let page = &mut page_file.0;
            // Old file, read in master page and header pages.
            let mut master_page_buf = vec![0u8; DEFAULT_PAGE_SIZE];
            page.read(&mut master_page_buf).await?;

            for (i, bits) in master_page_buf
                .splitn(2, |v| v.is_ascii_digit())
                .enumerate()
            {
                // fill master page.
                let v = u16::from_be_bytes([bits[0], bits[1]]);
                master_page.insert(i, v);

                // fill header page.
                let offset = Self::virtual_header_page_offset(i);
                if offset < page_len {
                    let mut header_page_buf = vec![0u8; DEFAULT_PAGE_SIZE];
                    page.seek(SeekFrom::Current(offset as i64)).await?;
                    page.read(&mut header_page_buf).await?;
                    header_page.insert(i, header_page_buf);
                }
            }
        }

        Ok(PartitionHandle {
            part_id,
            master_page,
            header_page,
            page_file,
        })
    }

    /// Writes the master page to disk, because the default page size of 4kb, so
    /// we put 1bit of bitmap as 2bits.
    async fn write_master_page(file: &mut File, master_page: &Vec<u16>) -> Result<()> {
        let mut page_buf = vec![0u8; DEFAULT_PAGE_SIZE];
        for index in 0..MAX_HEADER_PAGES {
            let v = master_page[index];
            page_buf.put_u16(v);
        }
        Ok(file.write_all(&page_buf).await?)
    }

    /// Writes the header page to disk.
    async fn write_header_page(file: &mut File, header_index: &Vec<u16>) -> Result<()> {
        Ok(())
    }

    #[inline]
    fn virtual_header_page_offset(header_index: usize) -> u64 {
        // Consider the layout if we had 4 data pages per header:
        // Offset (in pages):  0  1  2  3  4  5  6  7  8  9 10 11
        // Page Type:         [M][H][D][D][D][D][H][D][D][D][D][H]...
        // Header Index:          0              1              2
        // To get the offset in pages of a header page you should add 1 for
        // the master page, and then take the header index times the number
        // of data pages per header plus 1 to account for the header page
        // itself (in the above example this coefficient would be 5)
        let spacing_coeoff = DATA_PAGES_PER_HEADER + 1;
        ((1 + header_index * spacing_coeoff) * DEFAULT_PAGE_SIZE) as u64
    }

    #[inline]
    fn virtual_data_page_offset(page_num: usize) -> u64 {
        // Consider the layout if we had 4 data pages per header:
        // Offset (in pages):  0  1  2  3  4  5  6  7  8  9 10
        // Page Type:         [M][H][D][D][D][D][H][D][D][D][D]
        // Data Page Index:          0  1  2  3     4  5  6  7
        // To get the offset in pages of a given data page you should:
        // - add one for the master page
        // - add one for the first header page
        // - add how many other header pages precede the data page (found by floor
        //   dividing page num by data pages per header)
        // - add how many data pages precede the given data page (this works out
        //   conveniently to the page's page number)
        let other_headers = page_num / DATA_PAGES_PER_HEADER;
        ((2 + other_headers + page_num) * DEFAULT_PAGE_SIZE) as u64
    }

    /// Release all pages from partition for use.
    async fn release(&self) -> Result<()> {
        let idxs = self
            .master_page
            .iter()
            .copied()
            .filter(|&v| v != 0)
            .map(|v| v as usize)
            .collect::<Vec<_>>();

        for idx in idxs {
            self.header_page[idx]
                .iter()
                .filter(|&&v| v != 0u8)
                .for_each(|&v| {
                    self.release_page(idx * DATA_PAGES_PER_HEADER + (v as usize))
                        .unwrap();
                });
        }
        Ok(())
    }

    fn release_page(&self, page_num: usize) -> Result<()> {
        todo!()
    }
}

/// An implementation of a heap file, using a page group. Assumes data pages
/// are packed (but record lengths do not need to be fixed-length).
pub struct PageManager {
    // epoch: Epoch,
    guard: Mutex<()>,

    path: String,

    /// Page cache.
    cache: Lru<HeaderPage>,

    /// The page manager id.
    page_manager_id: u32,

    partitions: HashMap<usize, PartitionHandle>,

    /// Counter to generate new partition numbers.
    partition_counter: AtomicUsize,

    /// Partition to allocate new header pages in - may be different from
    /// partition from data pages.
    part_num: usize,

    /// The size of metadata of an empty data page.
    empty_page_metadata_size: usize,

    /// The first head page.
    page: HeaderPage,
}

/// private methods.
impl PageManager {
    /// Returns the number of data page entries in a header page.
    #[inline]
    fn header_entry_count(&self) -> usize {
        (self.effective_page_size() - HEADER_HEADER_SIZE) / DATA_HEADER_SIZE
    }

    /// Returns the effective page size.
    #[inline]
    fn effective_page_size(&self) -> usize {
        DEFAULT_PAGE_SIZE - RESERVED_SIZE
    }

    #[inline]
    pub(crate) fn set_empty_page_metadata_size(&mut self, empty_page_metadata_size: usize) {
        self.empty_page_metadata_size = self.effective_page_size() - empty_page_metadata_size;
    }

    #[inline]
    async fn inner_alloc_part(&mut self, part_num: usize) -> Result<usize> {
        let _guard = self.guard.lock();
        if self.partitions.contains_key(&part_num) {
            return Err(Error::Corrupted(format!(
                "allocate partition failed: partition number {} is exist.",
                part_num
            )));
        }
        let ph = PartitionHandle::create(part_num).await?;
        self.partitions.insert(part_num, ph);
        Ok(part_num)
    }

    async fn flush(&mut self) -> Result<()> {
        todo!()
    }
}

impl PageManager {
    /// PartNum: partition to allocate new header pages in (can be different
    /// partition  from data pages)
    /// - 0 represent redo or undo wal-log.
    /// - 1 represent table metadata.
    /// - 2 represent table indices.
    /// - .. represent table header/data page.
    pub fn new(part_num: usize, path: String) -> Self {
        PageManager {
            guard: Mutex::new(()),
            cache: Lru::with_capacity(10),
            page_manager_id: 0,
            part_num,
            empty_page_metadata_size: 0,
            page: HeaderPage::new(0, 0, true),
            partitions: HashMap::new(),
            partition_counter: AtomicUsize::new(0),
            path,
        }
    }

    /// Allocates a new partition, Returns number of new partition.
    pub(crate) async fn alloc_part(&mut self) -> Result<usize> {
        let part_num = self.partition_counter.fetch_add(1, Ordering::Release);
        self.inner_alloc_part(part_num).await
    }

    pub(crate) async fn alloc_part_with_num(&mut self, part_num: usize) -> Result<usize> {
        let num = self.partition_counter.load(Ordering::Acquire);
        let new_num = if part_num > num { part_num } else { num } + 1;

        if let Err(_) = self.partition_counter.compare_exchange(
            num,
            new_num,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            return Err(Error::Corrupted(format!(
                "allocate partitiion {} failed",
                part_num
            )));
        }

        self.inner_alloc_part(new_num).await
    }

    /// Release a partition from use.
    pub(crate) async fn release_part(&mut self, part_num: usize) -> Result<()> {
        let _guard = self.guard.lock();

        let part = self
            .partitions
            .remove(&part_num)
            .ok_or(Error::NotFound(format!(
                "release partitiion {} failed",
                part_num
            )))?;

        part.release().await?;

        fs::remove_file("path").await?;

        Ok(())
    }

    /// Allocates a new page, Return virtual page number of new page.
    pub(crate) async fn alloc_page(&self, page_num: usize) -> Result<usize> {
        todo!()
    }

    /// Release a page from use.
    pub(crate) async fn release_page(&self, page_num: usize) -> Result<()> {
        todo!()
    }

    /// Reads a page.
    pub(crate) async fn read_page(&self, page_num: usize) -> Result<Vec<u8>> {
        let mut data = vec![];
        self.read_page_to(page_num, &mut data).await?;
        Ok(data)
    }

    pub(crate) async fn read_page_to(&self, page_num: usize, target: &[u8]) -> Result<()> {
        todo!()
    }

    /// Writes to a page.
    pub(crate) async fn write_page(&self, data: &[u8]) -> Result<()> {
        todo!()
    }

    /// Checks if a page if allocated.
    pub(crate) async fn is_page_allocated(&self, page_num: usize) -> bool {
        todo!()
    }

    /// Gets partition from partition number.
    pub(crate) fn get_partition(&self, part_num: usize) -> Result<&PartitionHandle> {
        self.partitions
            .get(&part_num)
            .ok_or(Error::NotFound(format!("partition number {}", part_num)))
    }

    /// Gets page.
    pub async fn get_page(&self, page_num: usize) -> HeaderPage {
        let _guard = self.guard.lock();

        // Return header page if exist in page cache.
        if let Some(page) = self.cache.get(page_num) {
            return page;
        }

        // Read from disk if page file is exist and constructor `HeaderPage`.

        // Create new header page file and constructor `HeaderPage`.
        todo!()
    }

    pub fn get_page_with_space(&self, space: usize) -> HeaderPage {
        todo!()
    }

    pub fn get_num_data_pages(&self) -> usize {
        todo!()
    }

    pub(crate) fn part_num(&self) -> usize {
        self.part_num
    }
}

pub fn update_free_space(page: &HeaderPage, new_free_space: usize) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_part() {}

    #[test]
    fn test_alloc_page() {}

    #[test]
    fn test_write_page() {}

    #[test]
    fn test_read_page() {}
}
