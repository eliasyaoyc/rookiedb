use bytes::Bytes;

/// Buffer frame, containing information about the loaded page, wrapped around the
/// underlying byte array. Free frames use the index field to create a (single) linked
/// list between free frames.
pub struct Frame {
    contents: Bytes,
    index: isize,
    page_num: u64,
    dirty: bool,
    log_page: bool,
    pin_count: usize,
}

impl Frame {
    pub fn new() -> Self {
        todo!()
    }

    /// Pin buffer frame, cannot be evicted while pinned.  A "hit" happens when the
    /// buffer frame gets pinned.
    pub fn pin(&mut self) {
        self.pin_count += 1;
    }

    /// Unpin buffer frame.
    pub fn unpin(&mut self) {
        assert!(self.is_pinned(), "can't unpin unpinned frame.");
        self.pin_count -= 1;
    }

    /// Returns whether this frame is pinned.
    pub fn is_pinned(&self) -> bool {
        self.pin_count > 0
    }

    /// Returns whether this frame is valid.
    pub fn is_valid(&self) -> bool {
        self.index >= 0
    }

    /// Returns whether this frame's page has been freed.
    pub fn is_freed(&self) -> bool {
        self.index < 0 && self.index != isize::MAX
    }

    /// Invalidats the frame, flushing it if necessary.
    pub fn invalidate(&mut self) {
        if self.is_valid() {
            self.flush();
        }

        self.index = isize::MAX;
        self.contents.clear();
    }

    /// Marks the frame as free.
    pub fn set_free(&mut self, first_free_index: &mut isize) {
        assert!(!self.is_freed(), "can't free free frame.");
        let next_free_index = *first_free_index;
        *first_free_index = self.index;
        self.index = !next_free_index;
    }

    /// Marks the frame as used.
    pub fn set_used(&mut self, first_free_index: &mut isize) {
        assert!(self.is_freed(), "can't unfre used frame.");
        let index = *first_free_index;
        *first_free_index = !self.index;
        self.index = index;
    }

    /// Returns page number of this frame.
    pub fn page_num(&self) -> u64 {
        self.page_num
    }

    /// Flushes this buffer frame to disk, but dose not unload it.
    pub fn flush(&mut self) {
        self.pin();
        if !self.is_valid() || !self.dirty {
            return;
        }
        self.unpin();
    }
}
