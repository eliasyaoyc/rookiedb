use std::{
    borrow::Borrow, cmp::Ordering, collections::BTreeMap, marker::PhantomData, mem::MaybeUninit,
    ptr::NonNull, slice::SliceIndex,
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
    parent_idx: MaybeUninit<u16>,
    len: u16,
    keys: [MaybeUninit<K>; CAPACITY],
    vals: [MaybeUninit<V>; CAPACITY],
}

impl<'a, K: 'a, V: 'a> LeafNode<K, V> {
    unsafe fn init(this: *mut Self) {
        std::ptr::addr_of_mut!((*this).parent).write(None);
        std::ptr::addr_of_mut!((*this).len).write(0);
    }

    pub fn new_boxed() -> Box<LeafNode<K, V>> {
        let mut leaf = Box::new_uninit();
        unsafe {
            Self::init(leaf.as_mut_ptr());
            leaf.assume_init()
        }
    }
}

impl<K, V> LeafNode<K, V> {
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

    pub fn insert_leaf(&mut self, index: usize, key: K, val: V) {
        let len = self.len as usize;
        if len < CAPACITY {
            let new_len = len + 1;
            // insert fit.
            unsafe {
                let key_slice_ptr = self.key_area_mut(..new_len).as_mut_ptr();
                std::ptr::copy(
                    key_slice_ptr.add(index),
                    key_slice_ptr.add(index + 1),
                    new_len - index - 1,
                );
                (*key_slice_ptr.add(index)).write(key);

                let val_slice_ptr = self.val_area_mut(..new_len).as_mut_ptr();
                std::ptr::copy(
                    val_slice_ptr.add(index),
                    val_slice_ptr.add(index + 1),
                    new_len - index - 1,
                );
                (*val_slice_ptr.add(index)).write(val);
            }
            self.len = new_len as u16;
        }
        // needs split leaf node.
    }

    fn keys(&self) -> &[K] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.keys.get_unchecked(..self.len as usize)) }
    }

    fn vals(&self) -> &[V] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.vals.get_unchecked(..self.len as usize)) }
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

    fn get_val_by_index(&self, idx: usize) -> Option<&V> {
        assert!(idx < self.vals.len());
        unsafe {
            let v = self.vals.get_unchecked(idx).assume_init_ref();
            Some(v)
        }
    }
}

impl<K, V> Node<K, V> for LeafNode<K, V> {
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

    #[deprecated]
    fn descend(&self, height: usize, idx: usize) -> NodeRef<K, V> {
        unsafe {
            let edge = self.edges.get_unchecked(idx).assume_init();

            NodeRef {
                height,
                node: edge,
                _marker: PhantomData,
            }
        }
    }

    fn edge_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<BoxedNode<K, V>>], Output = Output>,
    {
        unsafe { self.edges.as_mut_slice().get_unchecked_mut(index) }
    }

    unsafe fn as_internal_unchecked<'a>(this: *mut dyn Node<K, V>) -> &'a InternalNode<K, V> {
        unsafe { &*(this as *mut InternalNode<K, V>) }
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

/// primitive methods.
impl<K, V> NodeRef<K, V> {
    fn move_to_leaf(self, k: &K) -> usize {
        // let mut this = self;
        // let mut index = 0;
        // loop {
        //     let node_ref = unsafe { this.node.as_ref() };
        //     if node_ref.is_leaf() {
        //         return index;
        //     }
        //     let internal = NodeRef::as_internal_unchecked(&this);
        //     let keys = internal.data.keys();

        //     if let Some(idx) = keys
        //         .iter()
        //         .enumerate()
        //         .find(|(_, &key)| k.cmp(key.borrow()) == Ordering::Less)
        //         .map(|(idx, _)| idx)
        //     {
        //         let edge = unsafe { internal.edges.get_unchecked(idx).assume_init()
        // };

        //         let new_node_ref = NodeRef {
        //             height: self.height - 1,
        //             node: edge,
        //             _marker: PhantomData,
        //         };

        //         index = idx;
        //         this = new_node_ref;
        //     }
        // }
        0
    }
}

impl<'a, K: 'a, V: 'a> NodeRef<K, V> {
    pub fn new_leaf() -> Self {
        Self {
            height: 0,
            node: NonNull::from(Box::leak(LeafNode::new_boxed())),
            _marker: PhantomData,
        }
    }

    pub fn search_node<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let internal = unsafe { &*Self::as_internal_ptr(self) };
        internal.get_key(key)
    }

    pub fn insert(&mut self, key: K, value: V) {
        let index = self.move_to_leaf(&key);
        let leaf = Self::as_leaf_mut_unchecked(self);
        leaf.insert_leaf(index, key, value);
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

    fn as_leaf_unchecked(this: &Self) -> &LeafNode<K, V> {
        unsafe { &*(this.node.as_ptr() as *mut LeafNode<K, V>) }
    }

    fn as_leaf_mut_unchecked(this: &mut Self) -> &mut LeafNode<K, V> {
        unsafe { &mut *(this.node.as_ptr() as *mut LeafNode<K, V>) }
    }

    fn as_internal_unchecked(this: &Self) -> &InternalNode<K, V> {
        unsafe { &*(this.node.as_ptr() as *mut InternalNode<K, V>) }
    }

    pub unsafe fn drop_key_val(&self) {}
}
