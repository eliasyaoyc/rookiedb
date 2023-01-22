use parking_lot::Mutex;

use super::{file::writer::PageWriter, HeaderPage};

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

/// An implementation of a heap file, using a page group. Assumes data pages
/// are packed (but record lengths do not need to be fixed-length).
pub struct PageManager {
    guard: Mutex<()>,

    /// The page manager id.
    page_manager_id: u32,

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

    pub(crate) fn set_empty_page_metadata_size(&mut self, empty_page_metadata_size: usize) {
        self.empty_page_metadata_size = self.effective_page_size() - empty_page_metadata_size;
    }
}

impl PageManager {
    /// PartNum: partition to allocate new header pages in (can be different
    /// partition  from data pages)
    /// - 0 represent table dir, such as header/data page.
    /// - 1 represent table metadata
    /// - 2 represent table indices
    pub fn new(part_num: usize) -> Self {
        PageManager {
            guard: Mutex::default(),
            page_manager_id: 0,
            page_writer: PageWriter::new(),
            part_num,
            empty_page_metadata_size: 0,
            page: HeaderPage::new(0, 0, true),
        }
    }

    pub fn get_page(&self, page_num: usize) -> HeaderPage {
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