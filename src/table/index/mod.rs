pub mod btree;
pub mod btree_builder;
pub mod node;

pub const B: usize = 6;
pub const CAPACITY: usize = 2 * B - 1;
pub const MIN_LEN_AFTER_SPLIT: usize = B - 1;
pub const KV_IDX_CENTER: usize = B - 1;
pub const EDGE_IDX_LEFT_OF_CENTER: usize = B - 1;
pub const EDGE_IDX_RIGHT_OF_CENTER: usize = B;