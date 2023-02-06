mod builder;

use std::{fmt, sync::Arc};

use super::array::{primitive_array::I32Array, ArrayBuilderImpl, ArrayImpl};

/// A collection of arrays.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataChunk {
    arrays: Arc<[ArrayImpl]>,
    cardinality: usize,
}

impl FromIterator<ArrayImpl> for DataChunk {
    fn from_iter<T: IntoIterator<Item = ArrayImpl>>(iter: T) -> Self {
        let arrays: Arc<[ArrayImpl]> = iter.into_iter().collect();
        let cardinality = arrays.first().map(ArrayImpl::len).unwrap_or(0);
        assert!(
            arrays.iter().map(|a| a.len()).all(|l| l == cardinality),
            "all arrays must have the same length"
        );
        DataChunk {
            arrays,
            cardinality,
        }
    }
}

impl FromIterator<ArrayBuilderImpl> for DataChunk {
    fn from_iter<T: IntoIterator<Item = ArrayBuilderImpl>>(iter: T) -> Self {
        iter.into_iter().map(|b| b.finish()).collect()
    }
}

impl DataChunk {
    pub fn single(item: i32) -> Self {
        todo!()
    }

    /// Return the number of rows in the chunk.
    pub fn cardinality(&self) -> usize {
        self.cardinality
    }
}

/// Print the data chunk as a pretty table.
impl fmt::Display for DataChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use comfy_table::Table;
        let mut table = Table::new();
        table.load_preset("||--+-++|    ++++++");
        for i in 0..self.cardinality() {
            let row: Vec<_> = self.arrays.iter().map(|a| a.get_to_string(i)).collect();
            table.add_row(row);
        }
        write!(f, "{}", table)
    }
}

impl fmt::Debug for DataChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

/// A chunk is a wrapper sturct for many data chunks.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chunk {
    data_chunks: Vec<DataChunk>,
    header: Option<Vec<String>>,
}

impl Chunk {}
