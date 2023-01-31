use std::{
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use self::marker::LeafOrInternal;

const B: usize = 6;
pub const CAPACITY: usize = 2 * B - 1;
pub const MIN_LEN_AFTER_SPLIT: usize = B - 1;
const KV_IDX_CENTER: usize = B - 1;
const EDGE_IDX_LEFT_OF_CENTER: usize = B - 1;
const EDGE_IDX_RIGHT_OF_CENTER: usize = B;

/// The root node of an owned tree.
pub type Root<K, V> = NodeRef<K, V, marker::LeafOrInternal>;

impl<K, V, Type> Copy for NodeRef<K, V, Type> {}
impl<K, V, Type> Clone for NodeRef<K, V, Type> {
    fn clone(&self) -> Self {
        *self
    }
}

pub struct NodeRef<K, V, Type> {
    height: usize,
    node: NonNull<LeafNode<K, V>>,
    phantom: PhantomData<Type>,
}

impl<K, V, Type> NodeRef<K, V, Type> {
    pub fn len(&self) -> usize {
        unsafe { usize::from((*Self::as_leaf_ptr(self)).len) }
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn reborrow(&self) -> NodeRef<K, V, Type> {
        NodeRef {
            height: self.height,
            node: self.node,
            phantom: PhantomData,
        }
    }

    fn as_leaf_ptr(this: &Self) -> *mut LeafNode<K, V> {
        this.node.as_ptr()
    }

    pub unsafe fn drop_key_val(&self) {}
}

pub enum ForceResult<Leaf, Internal> {
    Leaf(Leaf),
    Internal(Internal),
}

impl<K, V> NodeRef<K, V, LeafOrInternal> {
    pub fn force(
        self,
    ) -> ForceResult<NodeRef<K, V, marker::Leaf>, NodeRef<K, V, marker::Internal>> {
        if self.height == 0 {
            ForceResult::Leaf(NodeRef {
                height: self.height,
                node: self.node,
                phantom: PhantomData,
            })
        } else {
            ForceResult::Internal(NodeRef {
                height: self.height,
                node: self.node,
                phantom: PhantomData,
            })
        }
    }

    unsafe fn cast_to_leaf_unchecked(self) -> NodeRef<K, V, marker::Leaf> {
        assert!(self.height == 0);
        NodeRef {
            height: self.height,
            node: self.node,
            phantom: PhantomData,
        }
    }

    unsafe fn cast_to_internal_unchecked(self) -> NodeRef<K, V, marker::Internal> {
        assert!(self.height > 0);
        NodeRef {
            height: self.height,
            node: self.node,
            phantom: PhantomData,
        }
    }
}

impl<K, V> NodeRef<K, V, marker::Leaf> {
    pub fn new_leaf() -> Self {
        Self::from_new_leaf(LeafNode::new())
    }

    pub fn forget_type(self) -> NodeRef<K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            phantom: PhantomData,
        }
    }

    fn from_new_leaf(leaf: Box<LeafNode<K, V>>) -> Self {
        Self {
            height: 0,
            node: NonNull::from(Box::leak(leaf)),
            phantom: PhantomData,
        }
    }
}

impl<K, V> NodeRef<K, V, marker::Internal> {
    fn new_internal(child: Root<K, V>) -> Self {
        let mut new_node = InternalNode::new();
        new_node.edges[0].write(child.node);
        unsafe { NodeRef::from_new_internal(new_node, child.height + 1) }
    }

    unsafe fn from_new_internal(internal: Box<InternalNode<K, V>>, height: usize) -> Self {
        assert!(height > 0);

        let node = NonNull::from(Box::leak(internal)).cast();
        let mut this = NodeRef {
            height,
            node,
            phantom: PhantomData,
        };
        //    this.borrow_mut().correct_all_childrens_parent_links();
        this
    }

    fn correct_all_childrens_parent_links(&mut self) {
        let len = self.len();
        for i in 0..=len {
            assert!(i <= self.len());
        }
    }

    fn from_internal(node: NonNull<InternalNode<K, V>>, height: usize) -> Self {
        assert!(height > 0);
        NodeRef {
            height,
            node: node.cast(),
            phantom: PhantomData,
        }
    }

    pub fn forget_type(self) -> NodeRef<K, V, marker::LeafOrInternal> {
        NodeRef {
            height: self.height,
            node: self.node,
            phantom: PhantomData,
        }
    }

    fn as_internal_mut(&mut self) -> &mut InternalNode<K, V> {
        let ptr = Self::as_internal_ptr(self);
        unsafe { &mut *ptr }
    }

    fn as_internal_ptr(this: &Self) -> *mut InternalNode<K, V> {
        this.node.as_ptr() as *mut InternalNode<K, V>
    }
}

type BoxedNode<K, V> = NonNull<LeafNode<K, V>>;

struct LeafNode<K, V> {
    /// We want to be covariant in `K` and `V`.
    parent: Option<NonNull<InternalNode<K, V>>>,

    /// This node's index into the parent node's `edges` array.
    /// `*node.parent.edges[node.parent_idx]` should be the same thing as
    /// `node`. This is only guaranteed to be initialized when `parent` is
    /// non-null.
    parent_idx: MaybeUninit<u16>,

    len: u16,

    keys: [MaybeUninit<K>; CAPACITY],
    vals: [MaybeUninit<V>; CAPACITY],
}

impl<K, V> LeafNode<K, V> {
    /// Initializes a new `LeaftNode` in-place.
    unsafe fn init(this: *mut Self) {
        unsafe {
            ptr::addr_of_mut!((*this).parent).write(None);
            ptr::addr_of_mut!((*this).len).write(0);
        }
    }

    /// Creates a new boxed `LeafNode`.
    fn new() -> Box<Self> {
        unsafe {
            let mut leaf = Box::new_uninit();
            LeafNode::init(leaf.as_mut_ptr());
            leaf.assume_init()
        }
    }
}

struct InternalNode<K, V> {
    data: LeafNode<K, V>,

    edges: [MaybeUninit<BoxedNode<K, V>>; 2 * B],
}

impl<K, V> InternalNode<K, V> {
    /// Creates a new boxed `InternalNode`.
    fn new() -> Box<Self> {
        unsafe {
            let mut node = Box::<Self>::new_uninit();
            // We only need to initialize the data; the edges are MaybeUninit.
            LeafNode::init(ptr::addr_of_mut!((*node.as_mut_ptr()).data));
            node.assume_init()
        }
    }
}

pub mod marker {
    pub enum Leaf {}

    pub enum Internal {}

    pub enum LeafOrInternal {}
}
