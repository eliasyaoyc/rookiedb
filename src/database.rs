use std::collections::HashMap;

use crate::{
    catalog::schema::Schema,
    datatypes::record::{Record, RecordId},
    error::{Error, Result},
    table::Table,
    Options,
};

/// Database keeps track of transactions, tables and indices and delegates work
/// to its disk manager, buffer manager, lock manager and recovery manager.
pub struct Database {
    options: Options,
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn open(options: Options) -> Self {
        Self {
            options,
            tables: HashMap::new(),
        }
    }

    pub async fn create_table(&mut self, table_name: String, schema: Schema) -> Result<()> {
        if self.tables.contains_key(&table_name) {
            return Err(Error::Corrupted(format!(
                "table name {} already exist.",
                table_name
            )));
        }

        let table = Table::create(
            schema,
            format!("{}/{}", self.options.path, table_name),
            self.options.num_records_per_page,
        )
        .await?;

        self.tables.insert(table_name, table);
        Ok(())
    }

    // todo(improve): batchRecord instead of record.
    pub async fn insert(&self, table_name: &str, record: Record) -> Result<RecordId> {
        let table = self
            .tables
            .get(table_name)
            .ok_or(Error::NotFound(format!("table {}", table_name)))?;

        table.insert(record)
    }
}
