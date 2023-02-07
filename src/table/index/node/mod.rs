use std::{mem::MaybeUninit, ptr::NonNull};

use self::{internal::InternalNode, leaf::LeafNode};

pub mod internal;
pub mod leaf;

pub const B: usize = 6;
pub const CAPACITY: usize = 2 * B - 1;
pub const MIN_LEN_AFTER_SPLIT: usize = B - 1;
pub const KV_IDX_CENTER: usize = B - 1;
pub const EDGE_IDX_LEFT_OF_CENTER: usize = B - 1;
pub const EDGE_IDX_RIGHT_OF_CENTER: usize = B;

pub type Root<K, V> = NonNull<Node<K, V>>;
pub type BoxedNode<K, V> = NonNull<Node<K, V>>;

pub enum Node<K, V> {
    Leaf(LeafNode<K, V>),
    Internal(InternalNode<K, V>),
}
impl<K, V> Node<K, V> {
    pub fn new_leaf_boxed() -> Box<LeafNode<K, V>> {
        LeafNode::new_boxed()
    }

    pub fn new_internal_boxed() -> Box<InternalNode<K, V>> {
        InternalNode::new_boxed()
    }

    /// Returns node whether leaf node.
    pub fn is_leaf(&self) -> bool {
        if let Node::Leaf(_) = self {
            return true;
        }
        false
    }

    pub unsafe fn drop_key_val(&self) {
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
