use std::{io::ErrorKind, ptr::NonNull, usize, vec};

use bytes::BufMut;

use super::{
    manager::{
        virtual_header_page_offset, DATA_PAGES_PER_HEADER, DEFAULT_PAGE_SIZE, MAX_HEADER_PAGES,
    },
    marker, PageFile, PageRef,
};
use crate::{
    error::{Error, Result},
    table::{
        cache::PageCache,
        page::manager::{effective_page_size, virtual_data_page_offset, virtual_page_num},
    },
    utils::{bitmap::Bitmap, fs},
};

pub struct PartitionHandle {
    /// Partition to allocate new header pages in - may be different from
    /// partition from data pages.
    part_num: usize,

    /// Page cache.
    cache: PageCache,

    page_file: PageFile,

    /// The bitmap of master page.
    m_bitmap: Bitmap,

    /// The bitmap of header page.
    h_bitmaps: Vec<Bitmap>,

    /// The size of metadata of an empty data page.
    empty_page_metadata_size: usize,

    first_header: PageRef<marker::Header>,
}

impl PartitionHandle {
    /// Opens the OS file and loads master and header pages.
    pub async fn open(
        part_num: usize,
        root: &str,
        empty_page_metadata_size: usize,
    ) -> Result<Self> {
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
            cache: PageCache::with_capacity(10),
            page_file,
            m_bitmap,
            h_bitmaps: Vec::with_capacity(DATA_PAGES_PER_HEADER),
            empty_page_metadata_size,
            first_header: PageRef::new_header_page(0, 0, true),
        })
    }

    /// Allocates a new page in the partition.
    pub async fn alloc_page(&mut self) -> Result<usize> {
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

    pub async fn alloc_page_with_index(
        &mut self,
        header_index: usize,
        page_index: usize,
    ) -> Result<usize> {
        assert!(
            !self.h_bitmaps[header_index].exist(page_index as u32),
            "page {} in header {} already allocated.",
            header_index,
            page_index,
        );
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
    pub async fn release_data_pages(&mut self) -> Result<()> {
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
    pub async fn release_page(&mut self, page_num: usize) -> Result<()> {
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
    pub async fn read_page(&mut self, page_num: usize, output: &mut [u8]) -> Result<()> {
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
    pub async fn write_page(&mut self, page_num: usize, buf: &[u8]) -> Result<()> {
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
    pub fn write_master_page(bitmap: &Bitmap) -> Vec<u8> {
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
    pub fn write_header_page(bitmap: &Bitmap) -> Vec<u8> {
        let mut buf = vec![0u8; DEFAULT_PAGE_SIZE];
        (0..DEFAULT_PAGE_SIZE).for_each(|index| {
            let v = if bitmap.exist(index as u32) { 1u8 } else { 0u8 };
            buf.put_u8(v);
        });
        buf
    }

    /// Checks if page number is for an unallocated data.
    pub fn is_not_allocated_page(&self, page_num: usize) -> bool {
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

    /// Gets page.
    pub async fn get_page(&mut self, page_num: u64) -> Result<PageRef<marker::Data>> {
        // Return header page if exist in page cache.
        if let Some(entry) = self.cache.lookup(page_num) {
            // return page;
            assert!(
                entry.num() == page_num,
                "get page failure, expect {} but got {}",
                page_num,
                entry.num()
            );

            return Ok(unsafe { entry.page().cast_to_data_page_unchecked() });
        }

        assert!(
            !self.is_not_allocated_page(page_num as usize),
            "page {} is not allocated",
            page_num
        );

        let mut buf = Vec::with_capacity(DEFAULT_PAGE_SIZE);
        self.read_page(page_num as usize, &mut buf).await?;

        let mut data_page = PageRef::new_data_page();
        data_page.as_data_page_mut().fill(&buf);
        Ok(data_page)
    }

    pub async fn get_page_with_space(&self, required_space: usize) -> PageRef<marker::Data> {
        assert!(
            required_space > 0,
            "can't request nonpositive amount of space."
        );

        assert!(
            required_space < effective_page_size() - self.empty_page_metadata_size,
            "requesting page with more space than the size of page."
        );

        // todo (project4_part2): Update the following line.

        let page = Box::new(
            self.first_header
                .as_header_page()
                .load_page_with_space(required_space)
                .await,
        );

        PageRef::from_data_page(unsafe { NonNull::new_unchecked(Box::leak(page)) })
    }

    /// Returns how many data pages in current partition.
    pub fn get_num_data_pages(self) -> usize {
        self.first_header
            .as_header_page()
            .iter()
            .map(|page| page.data_page_nums as usize)
            .sum()
    }

    pub fn part_num(&self) -> usize {
        self.part_num
    }
}
