pub(crate) mod schema;
use std::collections::HashMap;

use self::schema::Schema;
use crate::error::{Error, Result};

pub struct Catalog {
    tables: HashMap<String, Schema>,
}

impl Catalog {
    pub fn get_table(&self, table_name: &str) -> Result<&Schema> {
        self.tables
            .get(table_name)
            .ok_or(Error::NotFound(format!("table {}", table_name)))
    }

    pub fn create_table(&mut self) -> Result<()> {
        todo!()
    }

    pub fn drop_table(&mut self) -> Result<()> {
        todo!()
    }

    pub fn alter_table(&mut self) -> Result<()> {
        todo!()
    }
}
