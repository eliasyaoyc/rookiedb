use std::{collections::HashMap, path::PathBuf, sync::Arc};

use crate::{
    catalog::schema::Schema,
    datatypes::record::{Record, RecordId},
    error::{Error, Result},
    table::{recovery::Recovery, Table},
    Options,
};

/// Database keeps track of transactions, tables and indices and delegates work
/// to its disk manager, buffer manager, lock manager and recovery manager.
pub struct Database {
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn open(options: Options) -> Self {
        // recovery from path, if has datanase related files.
        let handle = Recovery::new(Arc::new(options));
        todo!()
    }

    pub fn create_table(&mut self, table_name: String, schema: Schema) -> Result<()> {
        if self.tables.contains_key(&table_name) {
            return Err(Error::Corrupted(format!(
                "table name {} already exist.",
                table_name
            )));
        }

        self.tables.insert(table_name, Table::new(schema));
        Ok(())
    }

    // todo(improve): batchRecord instead of record.
    pub fn insert(&self, table_name: &str, record: Record) -> Result<RecordId> {
        let table = self
            .tables
            .get(table_name)
            .ok_or(Error::NotFound(format!("table {}", table_name)))?;

        table.insert(record)
    }
}
