use std::marker::PhantomData;

use super::node::{marker, NodeRef, Root};
use crate::datatypes::record::RecordId;

/// A B+ tree builder.
///
/// # Example
/// ```rust
/// let mut builder = TreeBuilder::new();
/// builder.build();
/// ```
pub struct TreeBuilder<K, V> {
    phantom: PhantomData<(K, V)>,
}

impl<K, V> TreeBuilder<K, V> {
    pub fn new() -> Self {
        unimplemented!()
    }

    pub fn build(self) -> Tree<K, V> {
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
pub struct Tree<K, V> {
    height: usize,
    root: Option<Root<K, V>>,
}

impl<K, V> Drop for Tree<K, V> {
    fn drop(&mut self) {
        drop(unsafe { (std::ptr::read(self)) }.into_iter())
    }
}

/// private methods.
impl<K, V> Tree<K, V> {}

impl<K, V> Tree<K, V> {
    /// Returns the value associated with `key`.
    pub fn get(&self, key: K) -> Option<RecordId> {
        // type check.

        // todo(project4_integration)

        // todo(project2)

        None
    }

    pub fn scan(&self, key: K) -> Option<Vec<RecordId>> {
        // todo(project4_integration)

        // todo(project2)

        None
    }

    /// Inserts a (key, rid) pair into a B+ tree. If the key already exists
    /// in the B+ tree, then the pair is not inserted and an exception is
    /// raised.
    pub fn put(&self, key: K, id: RecordId) {
        // todo(project4_integration)

        // todo(project2)
    }

    /// Deletes a (key, rid) pair from a B+ tree.
    pub fn remove(&self, k: K) {
        // todo(project4_integration)

        // todo(project2)
    }

    pub fn bulk_load(&self) {
        todo!()
    }
}

pub struct IntoIter<K, V> {
    phantom: PhantomData<(K, V)>,
}

impl<K, V> IntoIter<K, V> {
    fn dying_next(&mut self) -> Option<NodeRef<K, V, marker::LeafOrInternal>> {
        None
    }
}

impl<K, V> Drop for IntoIter<K, V> {
    fn drop(&mut self) {
        struct DropGuard<'a, K, V>(&'a mut IntoIter<K, V>);

        impl<'a, K, V> Drop for DropGuard<'a, K, V> {
            fn drop(&mut self) {
                while let Some(kv) = self.0.dying_next() {
                    unsafe { kv.drop_key_val() };
                }
            }
        }

        while let Some(kv) = self.dying_next() {
            let guard = DropGuard(self);

            unsafe { kv.drop_key_val() };

            std::mem::forget(guard);
        }
    }
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<K, V> IntoIterator for Tree<K, V> {
    type IntoIter = IntoIter<K, V>;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}
