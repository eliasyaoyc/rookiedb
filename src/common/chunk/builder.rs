use crate::common::array::ArrayBuilderImpl;

/// A helper struct to build a [`DataChunk`].
pub struct DataChunkBuilder {
    array_builders: Vec<ArrayBuilderImpl>,
    size: usize,
    capacity: usize,
}