use std::{
    borrow::Borrow, cmp::Ordering, collections::BTreeMap, marker::PhantomData, mem::MaybeUninit,
    ptr::NonNull,
};

use super::{B, CAPACITY};

pub trait Node<K, V> {
    fn split(&mut self);

    fn merge(&mut self);

    fn steal(&mut self, count: usize);

    fn is_leaf(&self) -> bool;

    fn len(&self) -> usize;
}

struct LeafNode<K, V> {
    parent: Option<NonNull<InternalNode<K, V>>>,
    parent_idx: u16,
    len: u16,
    keys: [MaybeUninit<K>; CAPACITY],
    vals: [MaybeUninit<V>; CAPACITY],
}

impl<K: Ord, V> LeafNode<K, V> {
    pub fn get_key<Q: ?Sized>(&self, k: &Q) -> (Ordering, usize)
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let keys = self.keys();
        for (idx, key) in keys.iter().enumerate() {
            match k.cmp(key.borrow()) {
                Ordering::Less => return (Ordering::Less, idx),
                Ordering::Equal => return (Ordering::Equal, idx),
                Ordering::Greater => {}
            }
        }

        (Ordering::Less, self.keys.len())
    }

    fn keys(&self) -> &[K] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.keys.get_unchecked(..self.len as usize)) }
    }

    fn vals(&self) -> &[V] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.vals.get_unchecked(..self.len as usize)) }
    }

    fn get_val_by_index(&self, idx: usize) -> Option<&V> {
        assert!(idx < self.vals.len());
        unsafe {
            let v = self.vals.get_unchecked(idx).assume_init_ref();
            Some(v)
        }
    }
}

impl<K: Ord, V> Node<K, V> for LeafNode<K, V> {
    fn split(&mut self) {
        todo!()
    }

    fn merge(&mut self) {
        todo!()
    }

    fn steal(&mut self, count: usize) {
        todo!()
    }

    fn is_leaf(&self) -> bool {
        true
    }

    fn len(&self) -> usize {
        usize::from(self.len)
    }
}

struct InternalNode<K, V> {
    data: LeafNode<K, V>,
    edges: [MaybeUninit<BoxedNode<K, V>>; 2 * B],
}

impl<K, V> InternalNode<K, V> {
    unsafe fn as_internal_unchecked<'a>(this: *mut dyn Node<K, V>) -> &'a InternalNode<K, V> {
        unsafe { &*(this as *mut InternalNode<K, V>) }
    }

    pub fn get_key<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let mut node = self;
        loop {
            match node.data.get_key(k) {
                (Ordering::Less, idx) => {
                    if self.is_leaf() {
                        return None;
                    }

                    unsafe {
                        let edge = node.edges.get_unchecked(idx).assume_init();
                        let edge_ref = edge.as_ref();
                        let internal = Self::as_internal_unchecked(edge.as_ptr());
                        if edge_ref.is_leaf() {
                            match internal.data.get_key(k) {
                                (Ordering::Equal, idx) => {
                                    return internal.data.get_val_by_index(idx)
                                }
                                _ => return None,
                            }
                        };

                        node = internal;
                    }
                }
                (Ordering::Equal, idx) => return self.data.get_val_by_index(idx),
                _ => {}
            }
        }
    }
}

impl<K, V> Node<K, V> for InternalNode<K, V> {
    fn split(&mut self) {
        todo!()
    }

    fn merge(&mut self) {
        todo!()
    }

    fn steal(&mut self, count: usize) {
        todo!()
    }

    fn is_leaf(&self) -> bool {
        false
    }

    fn len(&self) -> usize {
        unimplemented!()
    }
}

/// The root node of an owned tree.
pub type Root<K, V> = NodeRef<K, V>;
type BoxedNode<K, V> = NonNull<dyn Node<K, V>>;

pub struct NodeRef<K, V> {
    height: usize,
    node: BoxedNode<K, V>,
    _marker: PhantomData<(K, V)>,
}

impl<K, V> Copy for NodeRef<K, V> {}
impl<K, V> Clone for NodeRef<K, V> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K, V> NodeRef<K, V> {
    pub fn search_node<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let internal = unsafe { &*Self::as_internal_ptr(self) };
        internal.get_key(key)
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn reborrow(&self) -> Self {
        NodeRef {
            height: self.height,
            node: self.node,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        unsafe { self.node.as_ref().len() }
    }

    fn as_leaf_ptr(this: &Self) -> *mut LeafNode<K, V> {
        this.node.as_ptr() as *mut LeafNode<K, V>
    }

    fn as_internal_ptr(this: &Self) -> *mut InternalNode<K, V> {
        this.node.as_ptr() as *mut InternalNode<K, V>
    }

    pub unsafe fn drop_key_val(&self) {}
}
