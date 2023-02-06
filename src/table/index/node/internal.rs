pub struct InternalNode<K, V> {
    pub data: LeafNode<K, V>,
    pub edges: [MaybeUninit<BoxedNode<K, V>>; 2 * B],
}

impl<K, V> InternalNode<K, V> {
    pub fn new_boxed() -> Box<Self> {
        let mut internal = Box::new_uninit();
        unsafe {
            ::init(leaf.as_mut_ptr());
            internal.assume_init()
        }
    }

    pub fn edge_area_mut<I, Output: ?Sized>(&mut self, index: I) -> &mut Output
    where
        I: SliceIndex<[MaybeUninit<BoxedNode<K, V>>], Output = Output>,
    {
        unsafe { self.edges.as_mut_slice().get_unchecked_mut(index) }
    }
}
