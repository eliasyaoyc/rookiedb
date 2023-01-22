use std::{collections::HashMap, sync::atomic::AtomicUsize};

use parking_lot::Mutex;

use super::{
    cache::lru::Lru,
    page::{file::writer::PageWriter, HeaderPage},
};
use crate::error::Result;

/// The default size of the page is 4KB.
const DEFAULT_PAGE_SIZE: usize = 4 * 1024;

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

pub struct PartitionHandle {}

impl PartitionHandle {}

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

    /// The page writer.
    page_writer: PageWriter,

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
            page_writer: PageWriter::new(),
            part_num,
            empty_page_metadata_size: 0,
            page: HeaderPage::new(0, 0, true),
            partitions: HashMap::new(),
            partition_counter: AtomicUsize::new(0),
            path,
        }
    }

    pub(crate) async fn alloc_partition(&self, part_num: usize) -> Result<usize> {
        todo!()
    }

    pub(crate) async fn free_partition(&self, part_num: usize) -> Result<()> {
        todo!()
    }

    pub(crate) async fn alloc_page(&self, page_num: usize) -> Result<usize> {
        todo!()
    }

    pub(crate) async fn free_page(&self, page_num: usize) -> Result<()> {
        todo!()
    }

    pub(crate) async fn read_page(&self, page_num: usize) -> Result<Vec<u8>> {
        todo!()
    }

    pub(crate) async fn read_page_to(&self, page_num: usize, target: &[u8]) -> Result<()> {
        todo!()
    }

    pub(crate) async fn write_page(&self, data: &[u8]) -> Result<()> {
        todo!()
    }

    pub(crate) async fn is_page_allocated(&self, page_num: usize) -> bool {
        todo!()
    }

    pub(crate) fn get_partition(&self, part_num: usize) -> Result<PartitionHandle> {
        todo!()
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
mod tests {}
