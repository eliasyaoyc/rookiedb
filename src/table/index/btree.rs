use std::{
    borrow::Borrow, collections::BTreeMap, fmt::Debug, marker::PhantomData, ops::RangeBounds,
    ptr::NonNull,
};

use super::node::{leaf::LeafNode, Node, NodeRef, Root};
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
impl<K, V> BTree<K, V> {}

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
        let node = self.root.as_ref()?;
        node.search_node(key)
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
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord + Clone,
    {
        match self.root.as_mut() {
            None => {
                let mut leaf = Node::new_leaf_boxed();
                leaf.as_mut().insert_leaf(key, value);

                self.root = Some(NodeRef::from_node(leaf.downcast()));
                self.length += 1;
                None
            }
            Some(node) => {
                let replaced = node.insert_node(key, value);
                self.length += 1;
                replaced
            }
        }
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
    fn test_sample_get() {
        let mut btree = BTree::new();
        btree.insert(1, 1);
        btree.insert(2, 2);
        btree.insert(3, 3);
        btree.insert(4, 4);

        assert_eq!(btree.get(&1), Some(&1));
        assert_eq!(btree.get(&2), Some(&2));
        assert_eq!(btree.get(&3), Some(&3));
        assert_eq!(btree.get(&4), Some(&4));

        assert_eq!(btree.get(&5), None);
    }

    #[test]
    fn test_sample_remove() {
        let mut btree = BTree::new();
        btree.insert(1, 1);
        assert_eq!(btree.get(&1), Some(&1));

        let removed = btree.remove(&1);
        assert_eq!(removed, Some(1));

        assert_eq!(btree.get(&1), None);
    }

    #[test]
    fn test_sample_scan() {}
}
