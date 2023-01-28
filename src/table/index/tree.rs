/// A B+ tree builder.
///
/// # Example
/// ```rust
/// let mut builder = TreeBuilder::new();
/// builder.build();
/// ```
pub struct TreeBuilder {}

impl TreeBuilder {
    pub fn new() -> TreeBuilder {
        unimplemented!()
    }

    pub fn build(self) -> Tree {
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
pub struct Tree {}

/// private methods.
impl Tree {}

impl Tree {
    pub fn get(&self) -> Option<()> {
        todo!()
    }

    pub fn insert(&self) -> Option<()> {
        todo!()
    }

    pub fn scan(&self) -> Option<()> {
        todo!()
    }

    pub fn remove(&self) {
        todo!()
    }

    pub fn bulk_load(&self) {
        todo!()
    }

    pub fn iter(&self) -> BPlusTreeIter {
        unimplemented!()
    }

    pub fn iter_mut(&mut self) -> BPlusTreeIter {
        unimplemented!()
    }
}

pub struct BPlusTreeIter {}

pub struct BPlusTreeIterMut {}
