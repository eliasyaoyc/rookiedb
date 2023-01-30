use std::collections::LinkedList;

use crate::table::page::{marker, PageRef};

pub struct LruEntry {
    num: u64,
    page: PageRef<marker::HeaderOrData>,
}

impl LruEntry {
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

pub struct Lru {
    list: LinkedList<LruEntry>,
    capacity: usize,
}

impl Lru {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            list: LinkedList::new(),
            capacity,
        }
    }

    pub fn lookup(&self, page_num: u64) -> Option<LruEntry> {
        todo!()
    }

    pub fn add(&mut self, entry: LruEntry) {
        let len = self.list.len();
        if len + 1 >= self.capacity {
            self.list.pop_front();
        }
        self.list.push_back(entry);
    }
}
