use std::{borrow::Borrow, mem::MaybeUninit, slice::SliceIndex};

use super::{move_to_slice, slice_remove, BoxedNode, Node, CAPACITY};
use crate::table::index::node::{slice_insert, slice_shl, slice_shr};

pub struct InternalNode<K, V> {
    pub len: u16,
    pub keys: [MaybeUninit<K>; CAPACITY],
    pub edges: [MaybeUninit<BoxedNode<K, V>>; CAPACITY],
}

/// pritive methods.
impl<K, V> InternalNode<K, V> {
    fn insert_internal_fit(&mut self, key: K, edge: BoxedNode<K, V>)
    where
        K: Ord,
    {
        let len = self.len();

        assert!(len < CAPACITY);

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
        let mut new_internal = Self::new_boxed();
        let splitpoint = CAPACITY / 2;

        let len = self.len();

        move_to_slice(
            self.key_area_mut(splitpoint..),
            new_internal.key_area_mut(..len - splitpoint),
        );
        move_to_slice(
            self.edge_area_mut(splitpoint..),
            new_internal.edge_area_mut(..len - splitpoint),
        );

        new_internal.len = (len - splitpoint) as u16;
        self.len = splitpoint as u16;

        new_internal
    }

    pub fn merge(&mut self, right_internal: &mut InternalNode<K, V>, discriminator_key: K) {
        assert!(self.len() + right_internal.len() < CAPACITY);
        let len = self.len();
        let r_len = right_internal.len();
        unsafe {
            slice_insert(self.key_area_mut(..len + 1), len, discriminator_key);
        }

        move_to_slice(
            right_internal.key_area_mut(..r_len),
            self.key_area_mut(len + 1..len + r_len + 1),
        );

        move_to_slice(
            right_internal.edge_area_mut(..r_len),
            self.edge_area_mut(len + 1..len + r_len + 1),
        );

        self.len += r_len as u16;
    }

    fn len(&self) -> usize {
        usize::from(self.len)
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

    pub fn downcast(self) -> Box<Node<K, V>> {
        Box::new(Node::Internal(self))
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
    ) -> (Option<K>, Option<Box<InternalNode<K, V>>>)
    where
        K: Ord,
    {
        let len = self.len();
        if len < CAPACITY {
            self.insert_internal_fit(key, edge);
            self.len += 1;
            return (None, None);
        }

        let mut split_node = self.split_internal();
        split_node.as_mut().insert_internal_fit(key, edge);
        self.len += 1;

        let splitkey = unsafe { split_node.keys.get_unchecked(0).assume_init_read() };
        (Some(splitkey), Some(split_node))
    }

    pub fn remove_internal(&mut self, edge: BoxedNode<K, V>) {
        let len = self.len();

        let mut remove_idx = 0;

        while remove_idx <= self.len() && self.get_edge_by_index(remove_idx) != edge {
            remove_idx += 1;
        }

        if remove_idx == len {
            return;
        }

        unsafe {
            slice_remove(self.key_area_mut(..len), remove_idx);
            slice_remove(self.edge_area_mut(..len), remove_idx);
        }

        self.len -= 1;
    }

    pub fn steal_left(
        &mut self,
        left_internal: &mut InternalNode<K, V>,
        discriminator_key: K,
    ) -> Option<K> {
        let len = self.len();

        let l_len = left_internal.len();
        assert!(len + 1 < CAPACITY);

        unsafe {
            slice_shr(self.key_area_mut(..len), 1);
            slice_shr(self.edge_area_mut(..len), 1);
        }

        let replacement_key = unsafe {
            let uninit_key = left_internal.keys.get_unchecked_mut(l_len - 1);

            let key = uninit_key.assume_init_read();
            uninit_key.assume_init_drop();

            Some(key)
        };

        unsafe {
            self.keys.get_unchecked_mut(0).write(discriminator_key);
        }

        std::mem::swap(
            self.get_mut_edge_by_index(0),
            left_internal.get_mut_edge_by_index(left_internal.len()),
        );

        self.len += 1;
        left_internal.len -= 1;

        replacement_key
    }

    pub fn steal_right(
        &mut self,
        right_internal: &mut InternalNode<K, V>,
        discriminator_key: K,
    ) -> Option<K> {
        let len = self.len();

        assert!(len + 1 < CAPACITY);

        unsafe {
            self.keys.get_unchecked_mut(len).write(discriminator_key);
        }

        std::mem::swap(
            self.get_mut_edge_by_index(len + 1),
            right_internal.get_mut_edge_by_index(0),
        );

        let replacement_key = unsafe {
            let uninit_key = right_internal.keys.get_unchecked_mut(0);

            let key = uninit_key.assume_init_read();
            uninit_key.assume_init_drop();

            Some(key)
        };

        unsafe {
            slice_shl(right_internal.key_area_mut(..self.len()), 1);
            slice_shl(right_internal.edge_area_mut(..self.len()), 1);
        }

        self.len += 1;
        right_internal.len -= 1;

        replacement_key
    }
}
