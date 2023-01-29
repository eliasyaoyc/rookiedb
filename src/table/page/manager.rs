use std::{
    sync::atomic::{AtomicUsize, Ordering},
    usize, vec,
};

use dashmap::{mapref::one::RefMut, DashMap};
use parking_lot::Mutex;

use super::{partition::PartitionHandle, HeaderPage};
use crate::{
    error::{Error, Result},
    table::cache::lru::Lru,
    utils::fs,
};

/// The default size of the page is 4KB.
pub(crate) const DEFAULT_PAGE_SIZE: usize = 4 * 1024;

pub(crate) const MAX_HEADER_PAGES: usize = DEFAULT_PAGE_SIZE / 2;

pub(crate) const DATA_PAGES_PER_HEADER: usize = DEFAULT_PAGE_SIZE * 8;

/// The size of header in header pages.
/// 1-byte  : check page is valid.
/// 4-bytes : page group id.
/// 8-bytes : next header page offset.
pub(crate) const HEADER_HEADER_SIZE: usize = 13;

/// The size of header in data pages.
pub(crate) const DATA_HEADER_SIZE: usize = 10;

/// The size of data entry.
/// 8-bytes : data page offset.
/// 2-bytes : free space in current data page.
pub(crate) const DATA_ENTRY_SIZE: usize = 10;

/// Reserve 36 bytes on each page for bookkeeping for recovery
/// (used to store the pageLSN, and to ensure that a redo-only/undo-only log
/// record can fit on one page).
pub(crate) const RESERVED_SIZE: usize = 36;

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

    partitions: DashMap<usize, PartitionHandle>,

    /// Counter to generate new partition numbers.
    partition_counter: AtomicUsize,

    /// The size of metadata of an empty data page.
    empty_page_metadata_size: usize,
}

/// private methods.
impl PageManager {
    #[inline]
    pub(crate) fn set_empty_page_metadata_size(&mut self, empty_page_metadata_size: usize) {
        self.empty_page_metadata_size = effective_page_size() - empty_page_metadata_size;
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
        let ph = PartitionHandle::open(part_num, &self.path, self.empty_page_metadata_size).await?;
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
    pub fn new(path: String) -> Self {
        PageManager {
            guard: Mutex::new(()),
            cache: Lru::with_capacity(10),
            page_manager_id: 0,
            empty_page_metadata_size: 0,
            partitions: DashMap::new(),
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

        if self
            .partition_counter
            .compare_exchange(num, new_num, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Err(Error::Corrupted(format!(
                "allocate partitiion {} failed",
                part_num
            )));
        }

        self.inner_alloc_part(new_num).await
    }

    /// Release a partition from use.
    pub(crate) async fn release_part(&mut self, part_num: usize) -> Result<()> {
        let mut part = self
            .partitions
            .remove(&part_num)
            .ok_or(Error::NotFound(format!(
                "release partitiion {} failed",
                part_num
            )))?;

        part.1.release_data_pages().await?;

        fs::remove_file(format!("{}.{}", self.path, part_num)).await?;

        Ok(())
    }

    /// Allocates a new page, Return virtual page number of new page.
    pub(crate) async fn alloc_page_with_part(&self, part_num: usize) -> Result<u64> {
        let mut ph = self.get_partition(part_num)?;

        let page_num = ph.alloc_page().await?;

        Ok(virtual_page_num(part_num, page_num))
    }

    pub(crate) async fn alloc_page(&self, page: u64) -> Result<u64> {
        let (part_num, page_num) = (calculate_part_num(page), calculate_page_num(page));
        let (header_index, page_index) = (
            page_num / DATA_PAGES_PER_HEADER,
            page_num % DATA_PAGES_PER_HEADER,
        );

        let mut ph = self.get_partition(part_num)?;

        ph.alloc_page_with_index(header_index, page_index).await?;

        Ok(virtual_page_num(part_num, page_num))
    }

    /// Release a page from use.
    pub(crate) async fn release_page(&self, page: u64) -> Result<()> {
        let (part_num, page_num) = (calculate_part_num(page), calculate_page_num(page));

        let mut ph = self.get_partition(part_num)?;
        ph.release_page(page_num).await?;
        Ok(())
    }

    /// Checks if a page if allocated.
    pub(crate) async fn is_page_allocated(&self, page: u64) -> bool {
        let (part_num, page_num) = (calculate_part_num(page), calculate_page_num(page));

        match self.partitions.get(&part_num) {
            Some(handle) => handle.is_not_allocated_page(page_num),
            None => false,
        }
    }

    /// Gets partition from partition number.
    pub(crate) fn get_partition(&self, part_num: usize) -> Result<RefMut<usize, PartitionHandle>> {
        self.partitions
            .get_mut(&part_num)
            .ok_or(Error::NotFound(format!("partition number {}", part_num)))
    }

    /// Reads a page(page parameters is virtual offset).
    pub(crate) async fn read_page(&self, page: u64) -> Result<Vec<u8>> {
        let mut data = vec![];
        self.read_page_to(page, &mut data).await?;
        Ok(data)
    }

    pub(crate) async fn read_page_to(&self, page: u64, target: &mut [u8]) -> Result<()> {
        assert!(
            target.len() == DEFAULT_PAGE_SIZE,
            "read page expects a 4kb, but got {}",
            target.len()
        );

        let (part_num, page_num) = (calculate_part_num(page), calculate_page_num(page));
        let mut ph = self.get_partition(part_num)?;
        ph.read_page(page_num, target).await?;
        Ok(())
    }

    /// Writes to a page.
    pub(crate) async fn write_page(&self, page: u64, data: &[u8]) -> Result<()> {
        assert!(
            data.len() == DEFAULT_PAGE_SIZE,
            "write page expects a 4kb, but got {}",
            data.len()
        );

        let (part_num, page_num) = (calculate_part_num(page), calculate_page_num(page));
        let mut ph = self.get_partition(part_num)?;
        ph.write_page(page_num, data).await?;
        Ok(())
    }
}

/// Returns the number of data page entries in a header page.
#[inline]
pub(crate) fn header_entry_count() -> usize {
    (effective_page_size() - HEADER_HEADER_SIZE) / DATA_HEADER_SIZE
}

/// Returns the effective page size.
#[inline]
pub(crate) fn effective_page_size() -> usize {
    DEFAULT_PAGE_SIZE - RESERVED_SIZE
}

#[inline]
pub(crate) fn virtual_header_page_offset(header_index: usize) -> u64 {
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
pub(crate) fn virtual_data_page_offset(page_num: usize) -> u64 {
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

/// Gets partition number from virtual page number.
#[inline]
pub(crate) fn calculate_part_num(virtial_page_num: u64) -> usize {
    (virtial_page_num / 10000000000) as usize
}

/// Gets data page number from virtual page number.
#[inline]
pub(crate) fn calculate_page_num(virtial_page_num: u64) -> usize {
    (virtial_page_num % 10000000000) as usize
}

/// Returns the virtual page number given partition/data page number.
#[inline]
pub(crate) fn virtual_page_num(part_num: usize, page_num: usize) -> u64 {
    (part_num * 10000000000 + page_num) as u64
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
