use std::{borrow::Borrow, fmt::Debug, marker::PhantomData, ops::RangeBounds};

use super::node::{Node, Root};
use crate::error::{Error, Result};
/// A persistent B+ tree.
///
/// ```rust
/// let mut tree = BPlusTree::new();
/// tree.insert(1, 1);
/// tree.insert(2, 2);
/// ```
pub struct BTree<K, V> {
    root: Option<Root<K, V>>,
    length: usize,
}

// impl<K, V> Drop for BTree<K, V> {
//     fn drop(&mut self) {
//         drop(unsafe { std::ptr::read(self) }.into_iter())
//     }
// }

/// private methods.
impl<K, V> BTree<K, V> {
    fn entry<Q: ?Sized>(&self, k: &Q)
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
    }
}

impl<K, V> BTree<K, V> {
    pub fn new() -> Self {
        Self {
            root: None,
            length: 0,
        }
    }

    /// Returns the value associated with `key`.
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        // let node = self.root.as_ref()?;
        // node.search_node(key)
        todo!()
    }

    /// Returns btree whether contains this key.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.get(key).is_some()
    }

    /// Inserts a (key, rid) pair into a B+ tree. If the key already exists
    /// in the B+ tree, then the pair is not inserted and an exception is
    /// raised.
    pub fn insert(&mut self, key: K, value: V)
    where
        K: Ord,
    {
        if self.contains_key(&key) {
            return;
        }

        // match self.root {
        //     Some(node) => {
        //         node.reborrow().insert(key, value);
        //         self.length += 1;
        //     }
        //     None => {
        //         let mut leaf = NodeRef::new_leaf();
        //         leaf.insert(key, value);
        //         self.root = Some(leaf);
        //         self.length += 1;
        //     }
        // }
    }

    /// Deletes a (key, rid) pair from a B+ tree.
    pub fn remove<Q: ?Sized>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        // todo(project4_integration)

        // todo(project2)

        todo!()
    }

    pub fn scan<T: ?Sized, R>(&self, range: R) -> Option<Vec<V>>
    where
        T: Ord,
        K: Borrow<T> + Ord,
        R: RangeBounds<T>,
    {
        // todo(project4_integration)

        // todo(project2)

        None
    }
}

pub struct IntoIter<K, V> {
    phantom: PhantomData<(K, V)>,
}

impl<K, V> IntoIter<K, V> {
    fn dying_next(&mut self) -> Option<Node<K, V>> {
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

impl<K, V> IntoIterator for BTree<K, V> {
    type IntoIter = IntoIter<K, V>;
    type Item = (K, V);

    fn into_iter(self) -> Self::IntoIter {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut btree = BTree::new();
        btree.insert(1, 1);
    }
}
