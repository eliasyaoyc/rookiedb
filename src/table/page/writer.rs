use bumpalo::Bump;

pub struct PageWriter {
    allocator: Bump,
}

impl PageWriter {
    pub fn new(capacity: usize) -> PageWriter {
        Self {
            allocator: Bump::with_capacity(capacity),
        }
    }

    pub fn alloc(&self) {
        todo!()
    }

    pub fn dealloc() {
        todo!()
    }

    pub fn flush() {
        todo!()
    }
}
