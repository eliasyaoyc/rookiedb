use std::marker::PhantomData;

use super::btree::BTree;
use crate::catalog::schema::TableIndex;

/// A B+ tree builder.
///
/// # Example
/// ```rust
/// let mut builder = TreeBuilder::new();
/// builder.build();
/// ```
pub struct BTreeBuilder<K, V> {
    phantom: PhantomData<(K, V)>,
}

impl<K, V> BTreeBuilder<K, V> {
    pub fn new(index: &TableIndex) -> Self {
        unimplemented!()
    }

    pub fn finish(self) -> BTree<K, V> {
        unimplemented!()
    }
}
