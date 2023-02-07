use std::{borrow::Borrow, mem::MaybeUninit, slice::SliceIndex};

use super::{slice_remove, BoxedNode, CAPACITY};
use crate::table::index::node::slice_insert;

pub struct InternalNode<K, V> {
    pub keys: [MaybeUninit<K>; CAPACITY],
    pub edges: [MaybeUninit<BoxedNode<K, V>>; CAPACITY],
}

/// pritive methods.
impl<K, V> InternalNode<K, V> {
    fn insert_internal_fit(&mut self, key: K, edge: BoxedNode<K, V>)
    where
        K: Ord,
    {
        assert!(self.len() < CAPACITY);
        let len = self.len();
        let keys = self.keys();
        let mut left_idx_in_parent = 0;
        while left_idx_in_parent < len && key > keys[left_idx_in_parent] {
            left_idx_in_parent += 1;
        }

        if left_idx_in_parent < len && key == keys[left_idx_in_parent] {
            return;
        }

        unsafe {
            slice_insert(self.key_area_mut(..len + 1), left_idx_in_parent, key);
            slice_insert(self.edge_area_mut(..len + 1), left_idx_in_parent, edge);
        }
    }

    fn split_internal(&mut self) -> Box<InternalNode<K, V>> {
        todo!()
    }

    fn merge(&mut self, right_internal: &mut InternalNode<K, V>) {
        
    }

    fn len(&self) -> usize {
        self.keys.len()
    }

    fn keys(&self) -> &[K] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.keys.get_unchecked(..self.len())) }
    }

    fn key_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<K>], Output = Output>,
    {
        unsafe { self.keys.as_mut_slice().get_unchecked_mut(index) }
    }

    fn edge_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<BoxedNode<K, V>>], Output = Output>,
    {
        unsafe { self.edges.as_mut_slice().get_unchecked_mut(index) }
    }

    fn get_key_by_index(&self, idx: usize) -> &K {
        assert!(idx < self.keys.len());
        unsafe { self.keys.get_unchecked(idx).assume_init_ref() }
    }

    fn get_mut_key_by_index(&mut self, idx: usize) -> &mut K {
        assert!(idx < self.keys.len());
        unsafe { self.keys.get_unchecked_mut(idx).assume_init_mut() }
    }

    fn get_edge_by_index(&self, idx: usize) -> BoxedNode<K, V> {
        assert!(idx < self.edges.len());
        unsafe { self.edges.get_unchecked(idx).assume_init_read() }
    }

    fn get_mut_edge_by_index(&mut self, idx: usize) -> &mut BoxedNode<K, V> {
        assert!(idx < self.edges.len());
        unsafe { self.edges.get_unchecked_mut(idx).assume_init_mut() }
    }

    fn as_raw_ptr(&mut self) -> *mut InternalNode<K, V> {
        self as *mut InternalNode<K, V>
    }
}

impl<K, V> InternalNode<K, V> {
    pub fn new_boxed() -> Box<Self> {
        let internal = Box::new_uninit();
        unsafe { internal.assume_init() }
    }

    pub fn search_internal<Q: ?Sized>(&self, key: &Q) -> BoxedNode<K, V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let mut idx = 0;
        while idx < self.len() && key >= self.get_key_by_index(idx).borrow() {
            idx += 1;
        }
        self.get_edge_by_index(idx)
    }

    pub fn insert_internal(
        &mut self,
        key: K,
        edge: BoxedNode<K, V>,
    ) -> Option<Box<InternalNode<K, V>>>
    where
        K: Ord,
    {
        let len = self.len();
        if len < CAPACITY {
            self.insert_internal_fit(key, edge);
            return None;
        }

        let mut split_node = self.split_internal();
        split_node.as_mut().insert_internal_fit(key, edge);

        Some(split_node)
    }

    pub fn remove_internal(&mut self, edge: BoxedNode<K, V>) {
        let mut right_edge_idx = 0;
        let len = self.len();

        while right_edge_idx <= self.len() && self.get_edge_by_index(right_edge_idx) != edge {
            right_edge_idx += 1;
        }

        unsafe {
            slice_remove(self.key_area_mut(..len), right_edge_idx);
            slice_remove(self.edge_area_mut(..len), right_edge_idx);
        }
    }

    pub fn steal_left(&mut self, left_internal: &mut InternalNode<K, V>) {}

    pub fn steal_right(&mut self, right_internal: &mut InternalNode<K, V>) {}
}
