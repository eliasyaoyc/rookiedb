use std::{borrow::Borrow, mem::MaybeUninit, ptr::NonNull, slice::SliceIndex};

use super::{move_to_slice, slice_insert, slice_remove, slice_shr, Node, CAPACITY};
use crate::table::index::node::slice_shl;

/// [`Leaf`] represnet a leaf node in b+tree, used for stored key-value pairs.
pub struct LeafNode<K, V> {
    /// The num of key-value pair in this node.
    pub len: u16,
    pub keys: [MaybeUninit<K>; CAPACITY],
    pub vals: [MaybeUninit<V>; CAPACITY],

    pub prev: Option<NonNull<LeafNode<K, V>>>,
    pub next: Option<NonNull<LeafNode<K, V>>>,
}

/// private methods.
impl<K, V> LeafNode<K, V> {
    unsafe fn init(this: *mut Self) {
        std::ptr::addr_of_mut!((*this).len).write(0);
        std::ptr::addr_of_mut!((*this).prev).write(None);
        std::ptr::addr_of_mut!((*this).next).write(None);
    }

    fn insert_leaf_fit(&mut self, key: K, val: V) -> Option<V>
    where
        K: Ord,
    {
        let len = self.len();
        let keys = self.keys();
        let mut insertion_idx = 0;
        while insertion_idx < len && key > keys[insertion_idx] {
            insertion_idx += 1;
        }

        // if key is exists, replace the value.
        if insertion_idx < len && key == keys[insertion_idx] {
            let old_val = std::mem::replace(self.get_mut_val_by_index(insertion_idx), val);
            return Some(old_val);
        }

        unsafe {
            slice_insert(self.key_area_mut(..len + 1), insertion_idx, key);
            slice_insert(self.val_area_mut(..len + 1), insertion_idx, val)
        }
        None
    }

    fn split_leaf(&mut self) -> Box<LeafNode<K, V>> {
        let mut new_leaf = Self::new_boxed();
        let splitpoint = CAPACITY / 2;

        let len = self.len();

        move_to_slice(
            self.key_area_mut(splitpoint..),
            new_leaf.key_area_mut(..len - splitpoint),
        );
        move_to_slice(
            self.val_area_mut(splitpoint..),
            new_leaf.val_area_mut(..len - splitpoint),
        );

        new_leaf.len = (len - splitpoint) as u16;
        self.len = splitpoint as u16;

        self.link_self(new_leaf.as_mut());

        new_leaf
    }

    fn link_self(&mut self, new_leaf: &mut LeafNode<K, V>) {
        new_leaf.prev = Some(NonNull::from(unsafe { &*self.as_raw_ptr() }));
        new_leaf.next = self.next;
        self.next = Some(NonNull::from(unsafe { &*new_leaf.as_raw_ptr() }));
    }

    fn len(&self) -> usize {
        usize::from(self.len)
    }

    fn keys(&self) -> &[K] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.keys.get_unchecked(..self.len())) }
    }

    fn vals(&self) -> &[V] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.vals.get_unchecked(..self.len())) }
    }

    fn key_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<K>], Output = Output>,
    {
        unsafe { self.keys.as_mut_slice().get_unchecked_mut(index) }
    }

    fn val_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<V>], Output = Output>,
    {
        unsafe { self.vals.as_mut_slice().get_unchecked_mut(index) }
    }

    fn get_key_by_index(&self, idx: usize) -> &K {
        assert!(idx < self.keys.len());
        unsafe { self.keys.get_unchecked(idx).assume_init_ref() }
    }

    fn get_mut_key_by_index(&mut self, idx: usize) -> &mut K {
        assert!(idx < self.keys.len());
        unsafe { self.keys.get_unchecked_mut(idx).assume_init_mut() }
    }

    fn get_val_by_index(&self, idx: usize) -> &V {
        assert!(idx < self.vals.len());
        unsafe { self.vals.get_unchecked(idx).assume_init_ref() }
    }

    fn get_mut_val_by_index(&mut self, idx: usize) -> &mut V {
        assert!(idx < self.vals.len());
        unsafe { self.vals.get_unchecked_mut(idx).assume_init_mut() }
    }

    fn as_raw_ptr(&mut self) -> *mut LeafNode<K, V> {
        self as *mut LeafNode<K, V>
    }
}

impl<K, V> LeafNode<K, V> {
    pub fn new_boxed() -> Box<LeafNode<K, V>> {
        let mut leaf = Box::new_uninit();
        unsafe {
            Self::init(leaf.as_mut_ptr());
            leaf.assume_init()
        }
    }

    pub fn downcast(self) -> Box<Node<K, V>> {
        Box::new(Node::Leaf(self))
    }

    /// Search key from keys of leaf node.
    pub fn search_leaf<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let keys = self.keys();
        for (idx, k) in keys.iter().enumerate() {
            if k.borrow() == key {
                return Some(self.get_val_by_index(idx));
            }
        }
        None
    }

    pub fn insert_leaf(
        &mut self,
        key: K,
        val: V,
    ) -> (Option<V>, Option<K>, Option<Box<LeafNode<K, V>>>)
    where
        K: Ord,
    {
        let len = self.len();
        if len < CAPACITY {
            let old_val = self.insert_leaf_fit(key, val);
            self.len += 1;
            return (old_val, None, None);
        }
        let mut split_node = self.split_leaf();
        let old_val = split_node.as_mut().insert_leaf_fit(key, val);
        self.len += 1;

        let splitkey = unsafe { split_node.keys.get_unchecked(0).assume_init_read() };

        (old_val, Some(splitkey), Some(split_node))
    }

    pub fn remove_leaf(&mut self, key: &K)
    where
        K: Ord,
    {
        let len = self.len();
        let mut remove_idx = 0;
        while remove_idx < len && *key != self.keys()[remove_idx] {
            remove_idx += 1;
        }

        if remove_idx == len {
            return;
        }

        unsafe {
            slice_remove(self.key_area_mut(..len), remove_idx);
            slice_remove(self.val_area_mut(..len), remove_idx);
        }

        self.len -= 1;
    }

    pub fn steal_left(&mut self, left_leaf: &mut LeafNode<K, V>) -> Option<K> {
        let len = self.len();

        assert!(len < CAPACITY);

        unsafe {
            slice_shr(self.key_area_mut(..len), 1);
            slice_shr(self.val_area_mut(..len), 1);
        }
        std::mem::swap(
            self.get_mut_key_by_index(0),
            left_leaf.get_mut_key_by_index(left_leaf.len() - 1),
        );
        std::mem::swap(
            self.get_mut_val_by_index(0),
            left_leaf.get_mut_val_by_index(left_leaf.len() - 1),
        );

        self.len += 1;
        left_leaf.len -= 1;
        Some(unsafe { self.keys.get_unchecked(0).assume_init_read() })
    }

    pub fn steal_right(&mut self, right_leaf: &mut LeafNode<K, V>) -> Option<K> {
        let len = self.len();
        assert!(len < CAPACITY);

        std::mem::swap(
            self.get_mut_key_by_index(len),
            right_leaf.get_mut_key_by_index(0),
        );
        std::mem::swap(
            self.get_mut_val_by_index(len),
            right_leaf.get_mut_val_by_index(0),
        );

        unsafe {
            slice_shl(right_leaf.key_area_mut(..self.len()), 1);
            slice_shl(right_leaf.val_area_mut(..self.len()), 1);
        }

        self.len += 1;
        right_leaf.len -= 1;
        Some(unsafe { right_leaf.keys.get_unchecked(0).assume_init_read() })
    }

    pub fn merge(&mut self, right_leaf: &mut LeafNode<K, V>) {
        assert!(self.len() + right_leaf.len() < CAPACITY);
        let r_len = right_leaf.len();
        move_to_slice(
            right_leaf.key_area_mut(..r_len),
            self.key_area_mut(self.len()..self.len() + r_len),
        );

        move_to_slice(
            right_leaf.val_area_mut(..r_len),
            self.val_area_mut(self.len()..self.len() + r_len),
        );
        self.len += r_len as u16;
        self.next = right_leaf.next;
    }
}
