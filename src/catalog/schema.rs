use crate::{datatypes::record::Record, error::Result};

#[derive(Debug)]
pub struct Schema {}

impl Schema {
    /// Adds a new field to the schema. Returns the schema so that calls can be
    /// together.
    pub fn add(&mut self) {}

    pub fn estimated_size(&self) -> usize {
        todo!()
    }

    /// Verifies that a record matches the given schema. Performs the following
    /// implicit casts:
    /// - String's of the wrong size are cast to the expected size of the
    ///   schame.
    /// - Int's will be cast to floats if a float is expected.
    pub(crate) fn verify_record(&self, record: Record) -> Result<Record> {
        todo!()
    }
}
