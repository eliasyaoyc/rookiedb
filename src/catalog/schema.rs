use crate::{common::record::Record, error::Result};

pub type ColumnId = u16;
pub struct Schema {
    indices: Vec<TableIndex>,
}

impl Schema {
    /// Adds a new field to the schema. Returns the schema so that calls can be
    /// together.
    pub fn add(&mut self) {}

    /// Verifies that a record matches the given schema. Performs the following
    /// implicit casts:
    /// - String's of the wrong size are cast to the expected size of the
    ///   schame.
    /// - Int's will be cast to floats if a float is expected.
    pub fn verify_record(&self, record: Record) -> Result<Record> {
        todo!()
    }

    /// Returns all indicies associated table.
    pub fn get_indcies<'a, 'b>(&'a self) -> &'b [TableIndex]
    where
        'a: 'b,
    {
        &self.indices
    }

    pub fn estimated_size(&self) -> usize {
        todo!()
    }
}

pub struct TableIndex {
    /// Index name.
    pub name: String,

    /// The column id corresponding to the index.
    pub cols: Vec<ColumnId>,
}
