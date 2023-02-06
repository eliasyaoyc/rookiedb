/// [`Leaf`] represnet a leaf node in b+tree, used for stored key-value pairs.
pub struct Leaf<K, V> {
    pub parent: Option<NonNull<InternalNode<K, V>>>,
    pub parent_idx: MaybeUninit<u16>,
    /// The num of key-value pair in this node.
    pub len: u16,
    pub keys: [MaybeUninit<K>; CAPACITY],
    pub vals: [MaybeUninit<V>; CAPACITY],
}

impl<K, V> Leaf<K, V> {
    pub unsafe fn init(this: *mut Self) {
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

    pub fn keys(&self) -> &[K] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.keys.get_unchecked(..self.len as usize)) }
    }

    pub fn vals(&self) -> &[V] {
        unsafe { MaybeUninit::slice_assume_init_ref(self.vals.get_unchecked(..self.len as usize)) }
    }

    pub fn key_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<K>], Output = Output>,
    {
        unsafe { self.keys.as_mut_slice().get_unchecked_mut(index) }
    }

    pub fn val_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<V>], Output = Output>,
    {
        unsafe { self.vals.as_mut_slice().get_unchecked_mut(index) }
    }

    pub fn get_val_by_index(&self, idx: usize) -> Option<&V> {
        assert!(idx < self.vals.len());
        unsafe {
            let v = self.vals.get_unchecked(idx).assume_init_ref();
            Some(v)
        }
    }
}
