mod b_plus_metadata;
mod b_plus_node;
mod b_plus_tree;

pub trait Indexer {
    fn get(&self) -> Option<()>;

    fn put(&self) -> Option<()>;

    fn scan(&self) -> Option<()>;

    fn remove(&self);

    fn bulk_load(&self);
}
