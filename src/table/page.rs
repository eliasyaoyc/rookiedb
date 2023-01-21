use self::writer::PageWriter;

mod reader;
mod writer;

/// Todo(improvement): extend to 16kb?.
/// The default size of the page is 4KB.
const DEFAULT_PAGE_SIZE: usize = 4 * 1024;

/// The size of header in header pages.
const HEADER_HEADER_SIZE: usize = 13;

/// The size of header in data pages.
const DATA_HEADER_SIZE: usize = 10;

/// The size of data entry.
const DATA_ENTRY_SIZE: usize = 10;

/// Reserve 36 bytes on each page for bookkeeping for recovery
/// (used to store the pageLSN, and to ensure that a redo-only/undo-only log
/// record can fit on one page).
const RESERVED_SIZE: usize = 36;

/// An implementation of a heap file, using a page directory. Assumes data pages
/// are packed (but   record
///  lengths do not need to be fixed-length).
///
///  Header pages are layed out as follows:
///  - first byte: 0x1 to indicate valid allocated page
///  - next 4 bytes: page directory id
///  - next 8 bytes: page number of next header page, or -1 (0xFFFFFFFFFFFFFFFF)
///    if no next header page.
///  - next 10 bytes: page number of data page (or -1), followed by 2 bytes of
///    amount of free space
///  - repeat 10 byte entries
///
///  Data pages contain a small header containing:
///  - 4-byte page directory id
///  - 4-byte index of which header page manages it
///  - 2-byte offset indicating which slot in the header page its data page
///    entry resides
///
///  This header is used to quickly locate and update the header page when the
/// amount of free    space on the data page
///  changes, as well as ensure that we do not modify pages in other page
/// directories by accident.
///
///  The page directory id is a randomly generated 32-bit integer used to help
/// detect bugs (where    we attempt
///  to write to a page that is not managed by the page directory).
pub struct PageDirectory {
    /// The page directory id.
    page_directory_id: usize,

    /// The page writer.
    page_writer: PageWriter,

    /// Partition to allocate new header pages in - may be different from
    /// partition fro data pages.
    part_num: usize,

    /// The size of metadata of an empty data page.
    empty_page_metadata_size: usize,

    /// The first head page.
    page: Page,
}

/// private methods.
impl PageDirectory {
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

impl PageDirectory {
    pub fn new(
        page_writer: PageWriter,
        part_num: usize,
        page_num: usize,
        empty_page_metadata_size: usize,
    ) -> Self {
        PageDirectory {
            page_directory_id: 0,
            page_writer,
            part_num,
            empty_page_metadata_size,
            page: todo!(),
        }
    }

    pub fn get_page(&self, page_num: usize) -> Page {
        todo!()
    }

    pub fn get_page_with_space(&self, space: usize) -> Page {
        todo!()
    }

    pub fn get_num_data_pages(&self) -> usize {
        todo!()
    }

    pub(crate) fn part_num(&self) -> usize {
        self.part_num
    }
}

pub fn update_free_space(page: &Page, new_free_space: usize) {
    todo!()
}

/// Page represents a page loaded in memory (as opposed to the buffer frame it's
/// in). Wraps around buffer manager frames, and requests the page be loaded
/// into memory as necessary.
pub struct Page {}

pub struct DataPage {}

pub struct DataPageEntry {}

pub struct HeaderPage {}
