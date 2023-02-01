use std::collections::LinkedList;

use self::lru::LruEvictor;
use super::page::{marker, PageRef};

pub mod lru;

pub enum Evictor {
    Lru(LruEvictor),
}

pub struct CacheEntry {
    num: u64,
    page: PageRef<marker::HeaderOrData>,
}

impl CacheEntry {
    pub fn new(num: u64, page: PageRef<marker::HeaderOrData>) -> Self {
        Self { num, page }
    }

    pub fn num(&self) -> u64 {
        self.num
    }

    pub fn page(&self) -> PageRef<marker::HeaderOrData> {
        self.page
    }
}

pub struct PageCache {
    list: LinkedList<CacheEntry>,
    capacity: usize,
}

impl PageCache {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            list: LinkedList::new(),
            capacity,
        }
    }

    pub fn lookup(&self, _page_num: u64) -> Option<CacheEntry> {
        todo!()
    }

    pub fn add(&mut self, entry: CacheEntry) {
        let len = self.list.len();
        if len + 1 >= self.capacity {
            self.list.pop_front();
        }
        self.list.push_back(entry);
    }
}
