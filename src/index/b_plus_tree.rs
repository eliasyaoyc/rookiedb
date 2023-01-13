use super::Indexer;

/// A B+ tree builder.
///
/// # Example
/// ```rust
/// let mut builder = BPlusTreeBuilder::new();
/// builder.build();
/// ```
pub struct BPlusTreeBuilder {}

impl BPlusTreeBuilder {
    pub fn new() -> BPlusTreeBuilder {
        unimplemented!()
    }

    pub fn build(self) -> BPlusTree {
        unimplemented!()
    }
}

/// A persistent B+ tree.
///
/// ```rust
/// let mut tree = BPlusTree::new();
/// tree.insert();
/// tree.insert();
/// ```
pub struct BPlusTree {}

/// private methods.
impl BPlusTree {}

impl BPlusTree {
    pub fn iter(&self) -> BPlusTreeIter {
        unimplemented!()
    }

    pub fn iter_mut(&mut self) -> BPlusTreeIter {
        unimplemented!()
    }
}

impl Indexer for BPlusTree {
    fn get(&self) -> Option<()> {
        todo!()
    }

    fn put(&self) -> Option<()> {
        todo!()
    }

    fn scan(&self) -> Option<()> {
        todo!()
    }

    fn remove(&self) {
        todo!()
    }

    fn bulk_load(&self) {
        todo!()
    }
}

pub struct BPlusTreeIter {}

pub struct BPlusTreeIterMut {}