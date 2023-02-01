mod builder;

use std::{fmt, sync::Arc};

use super::array::{ArrayBuilderImpl, ArrayImpl};

/// A collection of arrays.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataChunk {
    arrays: Arc<[ArrayImpl]>,
    cardinality: usize,
}

impl FromIterator<ArrayImpl> for DataChunk {
    fn from_iter<T: IntoIterator<Item = ArrayImpl>>(iter: T) -> Self {
        todo!()
    }
}

impl FromIterator<ArrayBuilderImpl> for DataChunk {
    fn from_iter<T: IntoIterator<Item = ArrayBuilderImpl>>(iter: T) -> Self {
        todo!()
    }
}

/// Print the data chunk as a pretty table.
// impl fmt::Display for DataChunk {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         use comfy_table::Table;
//         let mut table = Table::new();
//         table.load_preset("||--+-++|    ++++++");
//         for i in 0..self.cardinality() {
//             let row: Vec<_> = self.arrays.iter().map(|a| a.get_to_string(i)).collect();
//             table.add_row(row);
//         }
//         write!(f, "{}", table)
//     }
// }

// impl fmt::Debug for DataChunk {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}", self)
//     }
// }

/// A chunk is a wrapper sturct for many data chunks.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Chunk {
    data_chunks: Vec<DataChunk>,
    header: Option<Vec<String>>,
}

impl Chunk {

}
