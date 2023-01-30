pub(crate) mod manager;
pub(crate) mod partition;
pub(crate) mod reader;

use std::{io::SeekFrom, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use async_fs::File;
use futures_lite::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use self::{manager::DEFAULT_PAGE_SIZE, reader::PageReader};
use crate::{datatypes::record::Record, error::Result};

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

pub mod marker {
    pub enum Header {}

    pub enum Data {}

    pub enum HeaderOrData {}
}

pub struct PageRef<Type> {
    is_header: bool,
    /// The pointer to the data or header node.
    page: NonNull<DataPage>,
    _marker: PhantomData<Type>,
}

impl<Type> Copy for PageRef<Type> {}
impl<Type> Clone for PageRef<Type> {
    fn clone(&self) -> Self {
        *self
    }
}

// Header/Data Page common methods.
impl<Type> PageRef<Type> {
    pub(crate) async fn get_idle_entry_num(&self) -> Result<usize> {
        todo!()
    }

    pub(crate) fn get_page_num(&self) -> u64 {
        todo!()
    }

    pub(crate) async fn insert_record(
        &self,
        entry_num: usize,
        record: crate::datatypes::record::Record,
    ) -> Result<()> {
        todo!()
    }

    pub(crate) async fn read_to_record(&self, offset: usize) -> Result<Record> {
        todo!()
    }
}

/// Convert to data page.
impl<'a, Type> PageRef<Type> {
    pub(crate) fn as_data_page(&self) -> &DataPage {
        let ptr = Self::as_data_page_ptr(self);
        unsafe { &*ptr }
    }

    pub(crate) fn as_data_page_mut(&mut self) -> &mut DataPage {
        let ptr = Self::as_data_page_ptr(self);
        unsafe { &mut *ptr }
    }

    pub(crate) fn into_data_page(self) -> &'a DataPage {
        let ptr = Self::as_data_page_ptr(&self);
        unsafe { &*ptr }
    }

    pub(crate) fn into_data_page_mut(mut self) -> &'a mut DataPage {
        let ptr = Self::as_data_page_ptr(&mut self);
        unsafe { &mut *ptr }
    }

    pub(crate) fn as_data_page_ptr(this: &Self) -> *mut DataPage {
        this.page.as_ptr()
    }
}

pub(crate) enum ForceResult<Header, Data> {
    Header(Header),
    Data(Data),
}
impl PageRef<marker::HeaderOrData> {
    /// Checks whether a page is an `Header` page or a `Data` page.
    pub(crate) fn force(self) -> ForceResult<PageRef<marker::Header>, PageRef<marker::Data>> {
        if self.is_header {
            ForceResult::Header(PageRef {
                is_header: true,
                page: self.page,
                _marker: PhantomData,
            })
        } else {
            ForceResult::Data(PageRef {
                is_header: false,
                page: self.page,
                _marker: PhantomData,
            })
        }
    }

    /// Unsafely asserts to the compiler the static information that this page
    /// is a `Header`
    pub(crate) unsafe fn cast_to_header_page_unchecked(self) -> PageRef<marker::Header> {
        PageRef {
            page: self.page,
            _marker: PhantomData,
            is_header: true,
        }
    }

    /// Unsafely asserts to the compiler the static information that this page
    /// is a `Data`
    pub(crate) unsafe fn cast_to_data_page_unchecked(self) -> PageRef<marker::Data> {
        PageRef {
            page: self.page,
            _marker: PhantomData,
            is_header: false,
        }
    }
}

// Header-Page proprietary methods.
impl PageRef<marker::Header> {
    pub(crate) fn new_header_page(page_num: usize, header_offset: usize, first: bool) -> Self {
        Self {
            page: NonNull::from(Box::leak(HeaderPage::new(page_num, header_offset, first))).cast(),
            _marker: PhantomData,
            is_header: true,
        }
    }

    /// Unpack a page reference that was packed as `PageRef::parent`.
    pub(crate) fn from_header_page(page: NonNull<HeaderPage>) -> Self {
        PageRef {
            page: page.cast(),
            _marker: PhantomData,
            is_header: true,
        }
    }

    pub(crate) fn forget_type(&self) -> PageRef<marker::HeaderOrData> {
        PageRef {
            page: self.page,
            _marker: PhantomData,
            is_header: true,
        }
    }

    pub(crate) fn as_header_page(&self) -> &HeaderPage {
        let ptr = Self::as_header_page_ptr(self);
        unsafe { &*ptr }
    }

    pub(crate) fn as_header_page_mut(&mut self) -> &mut HeaderPage {
        let ptr = Self::as_header_page_ptr(self);
        unsafe { &mut *ptr }
    }

    pub(crate) fn as_header_page_ptr(this: &Self) -> *mut HeaderPage {
        this.page.as_ptr() as *mut HeaderPage
    }
}

impl<'a> PageRef<marker::Header> {
    pub(crate) fn into_header_page(self) -> &'a HeaderPage {
        let ptr = Self::as_header_page_ptr(&self);
        unsafe { &*ptr }
    }

    pub(crate) fn into_header_page_mut(mut self) -> &'a mut HeaderPage {
        let ptr = Self::as_header_page_ptr(&mut self);
        unsafe { &mut *ptr }
    }
}

// Data-Page proprietary methods.
impl PageRef<marker::Data> {
    pub(crate) fn new_data_page() -> Self {
        Self {
            page: NonNull::from(Box::leak(DataPage::new())),
            _marker: PhantomData,
            is_header: false,
        }
    }

    /// Unpack a page reference that was packed as `PageRef::parent`.
    pub(crate) fn from_data_page(page: NonNull<DataPage>) -> Self {
        Self {
            page,
            _marker: PhantomData,
            is_header: false,
        }
    }

    pub(crate) fn forget_type(&self) -> PageRef<marker::HeaderOrData> {
        PageRef {
            page: self.page,
            _marker: PhantomData,
            is_header: false,
        }
    }

    /// Checks whether page is contains entry.
    pub(crate) fn contains(&self, entry_num: usize) -> bool {
        todo!()
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
    next: Option<NonNull<HeaderPage>>,
    header_offset: MaybeUninit<u16>,

    data_page_nums: u16,
    vals: [MaybeUninit<u8>; DEFAULT_PAGE_SIZE],
}

impl HeaderPage {
    pub(crate) unsafe fn init(this: *mut Self) {
        std::ptr::addr_of_mut!((*this).next).write(None);
        std::ptr::addr_of_mut!((*this).data_page_nums).write(0);
    }

    pub(crate) fn new(page_num: usize, header_offset: usize, first: bool) -> Box<Self> {
        unsafe {
            let mut hp = Box::new_uninit();
            HeaderPage::init(hp.as_mut_ptr());
            hp.assume_init()
        }
    }

    /// Gets and loads a page with the required free space.
    pub(crate) async fn load_page_with_space(&self, required_space: usize) -> DataPage {
        todo!()
    }

    pub(crate) fn iter(&self) -> HeaderPageIter {
        HeaderPageIter {
            next: self.next,
            header_offset: self.header_offset,
            data_page_nums: self.data_page_nums,
            vals: self.vals,
        }
    }
}

pub(crate) struct HeaderPageIter {
    next: Option<NonNull<HeaderPage>>,
    header_offset: MaybeUninit<u16>,

    data_page_nums: u16,
    vals: [MaybeUninit<u8>; DEFAULT_PAGE_SIZE],
}

impl Iterator for HeaderPageIter {
    type Item = Box<HeaderPage>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            None => None,
            Some(page) => unsafe {
                let p = &*page.as_ptr();

                self.next = p.next;

                Some(Box::from_raw(p as *const HeaderPage as *mut HeaderPage))
            },
        }
    }
}

pub struct DataPage {
    parent: Option<NonNull<HeaderPage>>,
    parent_idx: MaybeUninit<u16>,
    len: u16,
    vals: MaybeUninit<[u8; DEFAULT_PAGE_SIZE]>,
}

impl DataPage {
    /// Initializes a new `DataPage` in-place.
    pub(crate) unsafe fn init(this: *mut Self) {
        // As a general policy, we leave fields uninitialized if they can be, as this
        // should be both slightly faster and easier to track in Valgrind.

        // So parent_idx, vals are all MaybeUninit.
        std::ptr::addr_of_mut!((*this).parent).write(None);
        std::ptr::addr_of_mut!((*this).len).write(0);
    }

    /// Creates a new boxed `DataPage`.
    pub(crate) fn new() -> Box<Self> {
        unsafe {
            let mut dp = Box::new_uninit();
            DataPage::init(dp.as_mut_ptr());
            dp.assume_init()
        }
    }

    pub(crate) fn fill(&mut self, data: &[u8]) {
        assert!(data.len() == DEFAULT_PAGE_SIZE, "invalid data.");
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                self.vals.as_mut_ptr() as *mut _,
                DEFAULT_PAGE_SIZE,
            );
        }
    }
}

#[cfg(test)]
mod tests {}
