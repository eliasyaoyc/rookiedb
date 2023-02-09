pub mod internal;
pub mod leaf;

use std::{borrow::Borrow, mem::MaybeUninit, ptr::NonNull};

use self::{internal::InternalNode, leaf::LeafNode};

const B: usize = 6;
pub const CAPACITY: usize = 2 * B - 1;

pub type Root<K, V> = NodeRef<K, V>;
pub type BoxedNode<K, V> = NonNull<Node<K, V>>;

macro_rules! create_node_get_fn {
    ($name:ident, $param:ty, $ret:ty, $match:ident) => {
        pub fn $name(self: $param) -> Option<$ret> {
            match self {
                Node::$match(value) => Some(value),
                _ => None,
            }
        }
    };
}

pub enum Node<K, V> {
    Leaf(LeafNode<K, V>),
    Internal(InternalNode<K, V>),
}

impl<K, V> Node<K, V> {
    create_node_get_fn!(get_leaf, &Self, &LeafNode<K,V>,Leaf);

    create_node_get_fn!(get_leaf_mut, &mut Self, &mut LeafNode<K,V>, Leaf);

    create_node_get_fn!(get_internal, &Self, &InternalNode<K,V>, Internal);

    create_node_get_fn!(get_internal_mut, &mut Self, &mut InternalNode<K,V>, Internal);

    pub fn search_node(&self, k: K) {}

    pub fn new_leaf_boxed() -> Box<LeafNode<K, V>> {
        LeafNode::new_boxed()
    }

    pub fn new_internal_boxed() -> Box<InternalNode<K, V>> {
        InternalNode::new_boxed()
    }

    pub fn steal_left(&mut self, left_node: &mut Node<K, V>, discriminator_key: K) -> Option<K> {
        match &mut *self {
            Node::Leaf(leaf) => leaf.steal_left(left_node.get_leaf_mut().unwrap()),
            Node::Internal(internal) => {
                internal.steal_left(left_node.get_internal_mut().unwrap(), discriminator_key)
            }
        }
    }

    pub fn steal_right(&mut self, right_node: &mut Node<K, V>, discriminator_key: K) -> Option<K> {
        match &mut *self {
            Node::Leaf(leaf) => leaf.steal_right(right_node.get_leaf_mut().unwrap()),
            Node::Internal(internal) => {
                internal.steal_right(right_node.get_internal_mut().unwrap(), discriminator_key)
            }
        }
    }

    pub fn merge(&mut self, right_node: &mut Node<K, V>, discriminator_key: K) {
        match &mut *self {
            Node::Leaf(leaf) => leaf.merge(right_node.get_leaf_mut().unwrap()),
            Node::Internal(internal) => {
                internal.merge(right_node.get_internal_mut().unwrap(), discriminator_key)
            }
        }
    }

    /// Returns node whether leaf node.
    pub fn is_leaf(&self) -> bool {
        if let Node::Leaf(_) = self {
            return true;
        }
        false
    }

    pub fn is_underfull(&self) -> bool {
        match self {
            Node::Leaf(leaf) => leaf.len < ((CAPACITY - 1) / 2) as u16,
            Node::Internal(internal) => internal.len < (CAPACITY / 2) as u16,
        }
    }

    pub fn has_spcae_for_insert(&self) -> bool {
        match self {
            Node::Leaf(leaf) => leaf.len < (CAPACITY - 1) as u16,
            Node::Internal(internal) => internal.len < (CAPACITY - 1) as u16,
        }
    }

    pub fn has_space_for_removal(&self) -> bool {
        match self {
            Node::Leaf(leaf) => leaf.len > ((CAPACITY - 1) / 2) as u16,
            Node::Internal(internal) => internal.len > (CAPACITY / 2) as u16,
        }
    }

    pub unsafe fn drop_key_val(&self) {
        todo!()
    }
}

pub struct NodeRef<K, V> {
    node: NonNull<Node<K, V>>,
}

/// private methods.
impl<K, V> NodeRef<K, V> {
    fn reborrow(&self) -> Self {
        NodeRef { node: self.node }
    }

    fn descend_to_leaf<Q: ?Sized>(&self, key: &Q) -> Option<&LeafNode<K, V>>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let mut node = self.reborrow().node;
        loop {
            let node_ref = unsafe { node.as_ref() };
            if node_ref.is_leaf() {
                return node_ref.get_leaf();
            }
            let internal = node_ref.get_internal()?;
            node = internal.search_internal(key);
        }
    }

    fn descend_to_internal_leaf<Q: ?Sized>(
        &self,
        key: &Q,
    ) -> (Option<&mut InternalNode<K, V>>, Option<&mut LeafNode<K, V>>)
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        let mut parent = None;
        let mut node = self.reborrow().node;
        loop {
            let node_ref = unsafe { node.as_mut() };
            if node_ref.is_leaf() {
                return (parent, node_ref.get_leaf_mut());
            }
            let internal = node_ref.get_internal_mut();
            match internal {
                Some(internal_node) => {
                    node = internal_node.search_internal(key);
                    parent = Some(internal_node);
                }
                None => return (None, None),
            }
        }
    }

    fn insert_into_parent(
        &self,
        key: K,
        right: Box<InternalNode<K, V>>,
        parent_node: Option<&mut InternalNode<K, V>>,
    ) where
        K: Ord,
    {
        match parent_node {
            Some(parent) => {
                if let (splitkey, Some(splitnode)) =
                    parent.insert_internal(key, NonNull::from(Box::leak(right.downcast())))
                {
                    assert!(splitkey.is_some());
                    // self.insert_into_parent(splitkey.unwrap(), splitnode, parent_node);
                }
            }
            None => todo!(),
        }
    }
}

impl<K, V> NodeRef<K, V> {
    pub fn from_node(node: Box<Node<K, V>>) -> Self {
        Self {
            node: NonNull::from(Box::leak(node)),
        }
    }

    pub fn search_node<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        if let Some(leaf) = self.descend_to_leaf(key) {
            return leaf.search_leaf(key);
        }
        None
    }

    pub fn insert_node(&mut self, key: K, val: V) -> Option<V>
    where
        K: Ord + Clone,
    {
        let (parent, leaf) = self.descend_to_internal_leaf(&key);
        match leaf {
            Some(leaf) => {
                let (replaced, splitkey, splitnode) = leaf.insert_leaf(key, val);
                if let Some(splitnode) = splitnode {
                    assert!(splitkey.is_some());
                    // self.insert_into_parent(splitkey.unwrap(), splitnode, parent);
                }
                replaced
            }
            None => {
                assert!(parent.is_some());

                let mut leaf = Node::new_leaf_boxed();
                leaf.as_mut().insert_leaf(key.clone(), val);

                parent
                    .unwrap()
                    .insert_internal(key, NonNull::from(Box::leak(leaf.downcast())));

                None
            }
        }
    }

    pub fn remove_node(&mut self, key: K) -> Option<V> {
        todo!()
    }
}

/// Inserts a value into a slice of initialized elements followed by on
/// uninitialized element.
///
/// # Safety
/// The slice has more than `idx` elements.
pub unsafe fn slice_insert<T>(slice: &mut [MaybeUninit<T>], idx: usize, val: T) {
    unsafe {
        let len = slice.len();
        assert!(idx < len);
        let slice_ptr = slice.as_mut_ptr();
        if len > idx + 1 {
            std::ptr::copy(slice_ptr.add(idx), slice_ptr.add(idx + 1), len - idx - 1);
        }
        (*slice_ptr.add(idx)).write(val);
    }
}

/// Removes and returns a value from a slice of all initialized elements,
/// leaving behind one trailing uninitialized element.
///
/// # Safety
/// The slice has more than `idx` elements.
pub unsafe fn slice_remove<T>(slice: &mut [MaybeUninit<T>], idx: usize) -> T {
    unsafe {
        let len = slice.len();
        assert!(idx < len);
        let slice_ptr = slice.as_mut_ptr();
        let ret = (*slice_ptr.add(idx)).assume_init_read();
        std::ptr::copy(slice_ptr.add(idx + 1), slice_ptr.add(idx), len - idx - 1);
        ret
    }
}

/// Shifts the elements in a slice `distance` posititions to the left.
///
/// # Satefy
/// The slice has at least `distance` elements.
pub unsafe fn slice_shl<T>(slice: &mut [MaybeUninit<T>], distance: usize) {
    unsafe {
        let slice_ptr = slice.as_mut_ptr();
        std::ptr::copy(slice_ptr.add(distance), slice_ptr, slice.len() - distance);
    }
}

/// Shifts the elements in a slice `distance` posititions to the right.
///
/// # Satefy
/// The slice has at least `distance` elements.
pub unsafe fn slice_shr<T>(slice: &mut [MaybeUninit<T>], distance: usize) {
    unsafe {
        let slice_ptr = slice.as_mut_ptr();
        std::ptr::copy(slice_ptr, slice_ptr.add(distance), slice.len() - distance);
    }
}

/// Moves all values from a slice of initialized elements to a slice
/// of uninitialized elements, leaving behind `src` as all uninitialized.
/// Works like `dst.copy_from_slice(src)` but does not require `T` to be `Copy`.
pub fn move_to_slice<T>(src: &mut [MaybeUninit<T>], dst: &mut [MaybeUninit<T>]) {
    assert!(src.len() == dst.len());
    unsafe { std::ptr::copy(src.as_ptr(), dst.as_mut_ptr(), src.len()) }
}
