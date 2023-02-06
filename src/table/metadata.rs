use crate::catalog::schema::Schema;

pub struct TableMetadata {
    schema: Schema,
}

impl TableMetadata {
    pub fn new(schema: Schema) -> Self {
        Self { schema }
    }

    pub fn get_schema(&self) -> &Schema {
        &self.schema
    }

    pub fn flush(&self) -> Self {
        todo!()
    }
}
