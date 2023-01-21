use std::collections::LinkedList;

pub struct Lru<T> {
    list: LinkedList<T>,
    capacity: usize,
}

impl<T> Lru<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            list: LinkedList::new(),
            capacity,
        }
    }

    pub fn get(&self) -> Option<T> {
        todo!()
    }

    pub fn add(&mut self, t: T) {
        let len = self.list.len();
        if len + 1 >= self.capacity {
            self.list.pop_front();
        }
        self.list.push_back(t);
    }
}
