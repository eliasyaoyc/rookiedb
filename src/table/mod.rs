mod bg;
mod cache;
pub mod index;
mod manifest;
pub mod metadata;
pub mod page;
pub mod recover;
mod stats;

use std::collections::HashMap;

use self::{
    index::{btree::BTree, btree_builder::BTreeBuilder}, metadata::TableMetadata, page::partition::PartitionHandle,
    stats::TableStats,
};
use crate::{
    catalog::schema::Schema,
    common::record::{new_record_id, Record, RecordId},
    error::{Error, Result},
};

/// A Table represents a database table with which users can insert, get,
/// update and delete records.
///
/// # Persistence
/// Every table is persisted in its own `page_directory`, which interfaces with
/// buffer and disk to save it to disk.
///
/// A table can be loaded again by simply constructing it with the same
/// parameters.
///
/// # Storage Format
/// All pages are data pages - there are no header pages, because all metadata
/// is stored elsewhere (as rows in the _metadata.tables table). Every daa
/// which records inthe page are valid. The values of n and m are set to
/// maximize the number of records per page.
///
/// For example, here is a cartoon of what a table's file would look like if we
/// had 5-byte pages and 1-byte records:
///
///          +----------+----------+----------+----------+----------+ \
///   Page 0 | 1001xxxx | 01111010 | xxxxxxxx | xxxxxxxx | 01100001 |  |
///          +----------+----------+----------+----------+----------+  |
///   Page 1 | 1101xxxx | 01110010 | 01100100 | xxxxxxxx | 01101111 |  |- data
///          +----------+----------+----------+----------+----------+  |
///   Page 2 | 0011xxxx | xxxxxxxx | xxxxxxxx | 01111010 | 00100001 |  |
///          +----------+----------+----------+----------+----------+ /
///           \________/ \________/ \________/ \________/ \________/
///            bitmap     record 0   record 1   record 2   record 3
/// - The first page (Page 0) is a data page. The first byte of this data page
///   is a bitmap, and the next four bytes are each records. The first and
///   fourth bytes are set indicating that record 0 and record 3 ar valid.
///   Record 1 an record 2 ar invalid, so we ignore their contents. Similarly,
///   the last four bits of the bitmap are unused, so we ignore their contents.
/// - The second and third page (Page 1 and 2) are also data pages and are
///   formatted similar to Page 0.
///
/// When we add a record to a table, we add it to the very first free slot in
/// the table.
///
/// Some tables have large records. In order to efficiently handle tables with
/// large records (that still fit on a page), we format these tables a bit
/// differently, by giving each record a full page. Tables with full page
/// records do not have a bitmap. Instead, each allocated page is a single
/// record, and we indicate that a page dose not contain a record by simply
/// freeing the page.
///
/// In some cases, this behavior may be desriable even for small records(our
/// database only supports locking at the page level, so in cases where
/// tuple-level locks are necessary even at the cost of an I/O per tuple, a full
/// page record may be desirable), and may be explicitly toggled on with the
/// `set_full_page_records` methods.
pub struct Table {
    metadata: TableMetadata,

    indices: HashMap<String, BTree<i32, i32>>,

    /// The partition of table.
    part_handle: Box<PartitionHandle>,

    // The size (in bytes) of the bitmap found at the beginning of each data page.
    bitmap_size: usize,

    /// The number of records on each data page.
    num_records_per_page: usize,

    /// Statistics about the contents of the database.
    table_stats: TableStats,
}

/// Index associated method.
impl Table {
    async fn initialize_indices(&mut self) -> Result<()> {
        let indices = self.get_schema().get_indcies();
        let mut ans = HashMap::with_capacity(indices.len());
        for index in self.get_schema().get_indcies() {
            let builder = BTreeBuilder::<i32, i32>::new(index);
            let btree = builder.finish();

            ans.insert(index.name.clone(), btree);
        }

        self.indices = ans;

        Ok(())
    }

    async fn alter_index(&mut self) -> Result<()> {
        todo!()
    }

    async fn insert_index_entry(&self, record: &Record) -> Result<()> {
        todo!()
    }

    async fn remove_index_entry(&self, record: &Record) -> Result<()> {
        todo!()
    }
}
impl Table {
    /// Create a new table.
    pub async fn create(
        schema: Schema,
        part_handle: Box<PartitionHandle>,
        num_records_per_page: usize,
    ) -> Result<Self> {
        // todo enable cleanup and flush job.

        let mut table = Table {
            indices: HashMap::with_capacity(schema.get_indcies().len()),
            metadata: TableMetadata::new(schema),
            part_handle,
            num_records_per_page,
            table_stats: TableStats::new(),
            bitmap_size: 0,
        };

        table.initialize_indices().await?;
        Ok(table)
    }

    pub fn metadata(&self) -> &TableMetadata {
        &self.metadata
    }

    pub fn get_schema(&self) -> &Schema {
        self.metadata.get_schema()
    }

    pub fn num_records_per_page(&self) -> usize {
        self.num_records_per_page
    }

    // pub fn set_full_page_records(&mut self) {
    //     self.num_records_per_page = 1;
    //     self.bitmap_size = 0;
    //     self.page_manager
    //         .set_empty_page_metadata_size(self.schema.estimated_size());
    // }

    pub fn statistics(&self) -> &TableStats {
        &self.table_stats
    }

    pub fn get_part_num(&self) -> usize {
        self.part_handle.part_num()
    }

    /// Insert a record to this table and returns the record id of the newly
    /// added record. stats, freePageNums and numRecords are updated
    /// accordingly. The record is added to the first free slot of the first
    /// free page (if not exists, otherwise one is allocated). For examole,
    /// if the first free page has  bitmao 0b11101000, then the record is
    /// inserted into the page with index 3 and the bitmap is update to
    /// 0b11111000.
    pub async fn insert(&self, record: Record) -> Result<RecordId> {
        // Verify that the record whether valid. For example field value or field type.
        let schema = self.get_schema();
        let record = schema.verify_record(record)?;

        let page = self
            .part_handle
            .get_page_with_space(schema.estimated_size())
            .await;

        // Find the first empty slot in the bitmap.
        // entry number of the first free slot and store it in entry number;
        // and(2) we count the total number of entries on this page.
        let mut entry_num = page.get_idle_entry_num().await?;

        if self.num_records_per_page == 1 {
            entry_num = 0;
        }

        assert!(entry_num < self.num_records_per_page);

        // Insert the record and update the bitmap.
        page.insert_record(entry_num, &record).await?;

        // Insert the record to index.
        self.insert_index_entry(&record).await?;

        // Update the metadata.
        // todo stats ...
        Ok(new_record_id(page.get_page_num(), entry_num))
    }

    /// Retrieves a record from the table, throwing an exception if no such
    /// record exists.
    // todo needs get record from index of table.
    pub async fn get(&mut self, id: RecordId) -> Result<Record> {
        assert!(id.1 > 0 && id.1 < self.num_records_per_page);

        let page = self.part_handle.get_page(id.0).await?;
        if !page.contains(id.1) {
            return Err(Error::NotFound("record dose not exist.".to_owned()));
        }
        let offset = self.bitmap_size + (id.1 * self.get_schema().estimated_size());

        page.read_to_record(offset).await
    }

    /// Updates an existing record with new values and returns the existing
    /// record. stats is updated accordingly. An exception is thrown if
    /// recordId does not correspond to and existing record in the table.
    // todo needs get record from index of table.
    pub async fn update(&mut self, old_record_id: RecordId, updated: Record) -> Result<Record> {
        let entry_num = old_record_id.1;
        assert!(entry_num > 0 && entry_num < self.num_records_per_page);

        let record = self.get_schema().verify_record(updated)?;
        // If we're updating a record we'll need exclusive access to the page
        // it's on.
        // todo(project_part2): update the following line,

        let page = self.part_handle.get_page(old_record_id.0).await?;

        let old_record = self.get(old_record_id).await?;

        page.insert_record(entry_num, &record).await?;

        // Insert the record to index.
        self.insert_index_entry(&record).await?;

        // Update the metadata.
        // todo stats ...

        Ok(old_record)
    }

    /// Removes and returns the record specified bu recordId from the table and
    /// updates stats, freePageNums and numRecords as necessary. An
    /// exception is thrown if recordId dose not correspond to an existing
    /// record in the table.
    pub async fn remove(&mut self, id: RecordId) -> Result<Record> {
        assert!(id.1 > 0 && id.1 < self.num_records_per_page);

        let mut page = self.part_handle.get_page(id.0).await?;

        let record = page.remove_record(id.1).await?;

        let freed_space = if self.num_records_per_page == 1 {
            1
        } else {
            self.num_records_per_page - page.num_records() as usize
        } * self.get_schema().estimated_size();

        page.update_free_space(freed_space).await?;

        // Insert the record to index.
        self.remove_index_entry(&record).await?;

        // Update the metadata.
        // todo stats ...

        Ok(record)
    }
}

#[cfg(test)]
mod tests {}
