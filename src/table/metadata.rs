use crate::error::Result;

pub struct TableMetadata {
    //     indices:
}

impl TableMetadata {
    /// Returns a list of (recordId, metadata) pairs for all of the indices on
    /// the table `table_name` inside of _metadata.indices.
    /// Returns an empty list if the table does not exist, or if no indices are
    /// built on the table.
    pub fn get_table_indices(col_name: &str) -> Result<()> {
        todo!()
    }
}
