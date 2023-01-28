use std::{
    collections::HashMap,
    io::ErrorKind,
    sync::atomic::{AtomicUsize, Ordering},
    usize, vec,
};

use bytes::BufMut;
use parking_lot::Mutex;

use super::{
    cache::lru::Lru,
    page::{HeaderPage, PageFile},
};
use crate::{
    error::{Error, Result},
    utils::{bitmap::Bitmap, fs},
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
    part_num: usize,

    page_file: PageFile,

    /// The bitmap of master page.
    m_bitmap: Bitmap,

    /// The bitmap of header page.
    h_bitmaps: Vec<Bitmap>,
}

impl PartitionHandle {
    /// Opens the OS file and loads master and header pages.
    async fn open(part_num: usize, root: &str) -> Result<Self> {
        let part_name = format!("{}.{}", root, part_num);
        // Open the os file and loads master and header pages.
        let mut page_file = match fs::open(&part_name).await {
            Ok(file) => PageFile(file),
            Err(error) => {
                if ErrorKind::NotFound == error.kind() {
                    // File not exist and create new file.
                    PageFile(fs::create_file(part_name).await?)
                } else {
                    return Err(Error::IO(error));
                }
            }
        };

        let mut header_pages = Vec::with_capacity(MAX_HEADER_PAGES);

        let mut m_bitmap = Bitmap::new(MAX_HEADER_PAGES as u32);

        let page_len = page_file.0.metadata().await?.len();
        if page_len == 0 {
            // New file, write empty master page.
            page_file
                .write_f(|| Self::write_master_page(&m_bitmap))
                .await?;
        } else {
            // Old file, read in master page and header pages.
            let mut m_buf = vec![0u8; DEFAULT_PAGE_SIZE];
            page_file.read(&mut m_buf).await?;

            for (index, bits) in m_buf.splitn(2, |v| v.is_ascii_digit()).enumerate() {
                // fill master page.
                if u16::from_be_bytes([bits[0], bits[1]]) == 1 {
                    m_bitmap.set(index as u32);

                    // fill header page.
                    let offset = virtual_header_page_offset(index);
                    if offset < page_len {
                        let mut h_buf = vec![0u8; DEFAULT_PAGE_SIZE];
                        page_file.read_from(offset, &mut h_buf).await?;
                        header_pages.insert(index, h_buf);
                    }
                }
            }
        }

        Ok(PartitionHandle {
            part_num,
            page_file,
            m_bitmap,
            h_bitmaps: Vec::with_capacity(DATA_PAGES_PER_HEADER),
        })
    }

    /// Allocates a new page in the partition.
    async fn alloc_page(&mut self) -> Result<usize> {
        match self.m_bitmap.vacance() {
            None => Err(Error::Corrupted(
                "partition has reached max size.".to_owned(),
            )),
            Some(h) => match self.h_bitmaps[h as usize].vacance() {
                None => Err(Error::Corrupted(
                    "header page not has free space.".to_owned(),
                )),
                Some(p) => self.alloc_page_with_index(h as usize, p as usize).await,
            },
        }
    }

    async fn alloc_page_with_index(
        &mut self,
        header_index: usize,
        page_index: usize,
    ) -> Result<usize> {
        if self.h_bitmaps[header_index].exist(page_index as u32) {
            return Err(Error::Corrupted(format!(
                "page {} in header {} already allocated.",
                header_index, page_index
            )));
        }
        self.h_bitmaps[header_index].set(page_index as u32);
        self.m_bitmap.set(header_index as u32);

        let page_num = page_index + header_index * DATA_PAGES_PER_HEADER;

        let _vpn = virtual_page_num(self.part_num, page_num);

        self.page_file
            .write_to_f(0, || Self::write_master_page(&self.m_bitmap))
            .await?;
        self.page_file
            .write_to_f(virtual_header_page_offset(header_index), || {
                Self::write_header_page(&self.h_bitmaps[header_index])
            })
            .await?;
        Ok(page_num)
    }

    /// Release all data pages from partition for use.
    async fn release_data_pages(&mut self) -> Result<()> {
        let mut needs_freed_page_idx = vec![];
        let mut needs_freed_header_idx = vec![];
        for h in self.m_bitmap.iter() {
            for d in self.h_bitmaps[h as usize].iter() {
                needs_freed_page_idx.push(h as usize * DATA_PAGES_PER_HEADER + d as usize);
            }
            needs_freed_header_idx.push(h);
        }

        for idx in needs_freed_page_idx {
            self.release_page(idx).await?;
        }

        for idx in needs_freed_header_idx {
            self.m_bitmap.clear(idx);
        }

        Ok(())
    }

    /// Releases a page in partition from use.
    async fn release_page(&mut self, page_num: usize) -> Result<()> {
        let (header_index, page_index) = (
            (page_num / DATA_PAGES_PER_HEADER),
            (page_num % DATA_PAGES_PER_HEADER),
        );

        assert!(
            self.h_bitmaps[header_index].exist(page_index as u32),
            "can't release unallocated page."
        );

        let _vpn = virtual_page_num(self.part_num, page_num);

        // todo txn.

        // clear data page.
        self.write_page(page_num, &vec![0u8; DEFAULT_PAGE_SIZE])
            .await?;

        self.h_bitmaps[header_index].clear(page_index as u32);

        self.page_file
            .write_to_f(0, || Self::write_master_page(&self.m_bitmap))
            .await?;
        self.page_file
            .write_to_f(virtual_header_page_offset(header_index), || {
                Self::write_header_page(&self.h_bitmaps[header_index])
            })
            .await?;
        Ok(())
    }

    /// Reads in a data page. Assumes that the partition lock is held.
    async fn read_page(&mut self, page_num: usize, output: &mut [u8]) -> Result<()> {
        assert!(
            !self.is_not_allocated_page(page_num),
            "page {} is not allocated",
            page_num
        );

        self.page_file
            .read_from(virtual_data_page_offset(page_num), output)
            .await?;
        Ok(())
    }

    /// Writes to a data page. Assumes that the partition lock is held.
    async fn write_page(&mut self, page_num: usize, buf: &[u8]) -> Result<()> {
        assert!(
            !self.is_not_allocated_page(page_num),
            "page {} is not allocated",
            page_num
        );

        self.page_file
            .write_to(virtual_data_page_offset(page_num), buf)
            .await?;

        Ok(())
    }

    /// Writes the master page to disk, because the default page size of 4kb, so
    /// we put 1bit of bitmap as 2bits.
    fn write_master_page(bitmap: &Bitmap) -> Vec<u8> {
        let mut buf = vec![0u8; DEFAULT_PAGE_SIZE];
        (0..MAX_HEADER_PAGES).for_each(|index| {
            let v = if bitmap.exist(index as u32) {
                1u16
            } else {
                0u16
            };
            buf.put_u16(v);
        });
        buf
    }

    /// Writes the header page to disk.
    fn write_header_page(bitmap: &Bitmap) -> Vec<u8> {
        let mut buf = vec![0u8; DEFAULT_PAGE_SIZE];
        (0..DEFAULT_PAGE_SIZE).for_each(|index| {
            let v = if bitmap.exist(index as u32) { 1u8 } else { 0u8 };
            buf.put_u8(v);
        });
        buf
    }

    /// Checks if page number is for an unallocated data.
    fn is_not_allocated_page(&self, page_num: usize) -> bool {
        let (header_index, page_index) = (
            (page_num / DATA_PAGES_PER_HEADER),
            (page_num % DATA_PAGES_PER_HEADER),
        );

        if header_index >= MAX_HEADER_PAGES
            || !self.m_bitmap.exist(header_index as u32)
            || !self.h_bitmaps[header_index].exist(page_index as u32)
        {
            return true;
        }

        false
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
        let ph = PartitionHandle::open(part_num, &self.path).await?;
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

        let mut part = self
            .partitions
            .remove(&part_num)
            .ok_or(Error::NotFound(format!(
                "release partitiion {} failed",
                part_num
            )))?;

        part.release_data_pages().await?;

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
        if let Some(page) = self.cache.lookup(page_num) {
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

/// Returns the number of data page entries in a header page.
#[inline]
fn header_entry_count() -> usize {
    (effective_page_size() - HEADER_HEADER_SIZE) / DATA_HEADER_SIZE
}

/// Returns the effective page size.
#[inline]
fn effective_page_size() -> usize {
    DEFAULT_PAGE_SIZE - RESERVED_SIZE
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

/// Gets partition number from virtual page number.
#[inline]
fn calculate_part_num(virtial_page_num: u64) -> usize {
    (virtial_page_num / 10000000000) as usize
}

/// Gets data page number from virtual page number.
#[inline]
fn calculate_page_num(virtial_page_num: u64) -> usize {
    (virtial_page_num % 10000000000) as usize
}

/// Returns the virtual page number given partition/data page number.
#[inline]
fn virtual_page_num(part_num: usize, page_num: usize) -> u64 {
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
