use std::collections::HashMap;

use crate::{
    catalog::schema::Schema,
    common::record::{Record, RecordId},
    error::{Error, Result},
    options::Options,
    table::{
        page::{page_directory::PageDirectory, partition::PartitionHandle},
        Table,
    },
};

/// Database keeps track of transactions, tables and indices and delegates work
/// to its disk manager, buffer manager, lock manager and recovery manager.
pub struct Database {
    options: Options,
    page_directory: PageDirectory,
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn open(options: Options) -> Self {
        let page_directory = PageDirectory::new(options.path.clone());

        Self {
            options,
            tables: HashMap::new(),
            page_directory,
        }
    }

    pub async fn create_table(&mut self, table_name: String, schema: Schema) -> Result<()> {
        if self.tables.contains_key(&table_name) {
            return Err(Error::Corrupted(format!(
                "table name {} already exist.",
                table_name
            )));
        }

        let part_num = self.page_directory.alloc_part().await?;

        // todo optimize.
        let ph = unsafe {
            let ptr = self.page_directory.get_partition(part_num)?.value() as *const PartitionHandle
                as *mut PartitionHandle;
            Box::from_raw(ptr)
        };

        let table = Table::create(schema, ph, self.options.num_records_per_page).await?;

        self.tables.insert(table_name, table);
        Ok(())
    }

    // todo(improve): batchRecord instead of record.
    pub async fn insert(&self, table_name: &str, record: Record) -> Result<RecordId> {
        let table = self
            .tables
            .get(table_name)
            .ok_or(Error::NotFound(format!("table {}", table_name)))?;

        table.insert(record).await
    }
}
