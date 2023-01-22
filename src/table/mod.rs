mod cache;
mod cleanup;
mod flush;
mod index;
mod manifest;
pub(crate) mod page;
mod page_manager;
pub(crate) mod recover;
mod stats;

use parking_lot::RwLock;

use self::{page_manager::PageManager, stats::TableStats};
use crate::{
    catalog::schema::Schema,
    datatypes::record::{Record, RecordId},
    error::Result,
    utils::fs,
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
    guard: RwLock<()>,

    /// The schema of the table.
    schema: Schema,

    /// The page group of the table.
    page_manager: PageManager,

    /// The number of records on each data page.
    num_records_per_page: usize,

    /// Statistics about the contents of the database.
    table_stats: TableStats,
}

impl Table {
    /// Create a new table.
    pub async fn create(
        schema: Schema,
        table_dir: String,
        num_records_per_page: usize,
    ) -> Result<Self> {
        fs::create_dir(&table_dir).await?;

        let page_manager = PageManager::new(0, table_dir.clone());

        // todo enable cleanup and flush job.

        let table = Table {
            guard: RwLock::default(),
            schema,
            page_manager,
            num_records_per_page,
            table_stats: TableStats::new(),
        };
        Ok(table)
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
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
        &&self.table_stats
    }

    pub fn get_part_num(&self) -> usize {
        self.page_manager.part_num()
    }

    /// Insert a record to this table and returns the record id of the newly
    /// added record. stats, freePageNums and numRecords are updated
    /// accordingly. The record is added to the first free slot of the first
    /// free page (if not exists, otherwise one is allocated). For examole,
    /// if the first free page has  bitmao 0b11101000, then the record is
    /// inserted into the page with index 3 and the bitmap is update to
    /// 0b11111000.
    pub fn insert(&self, record: Record) -> Result<RecordId> {
        // Verify that the record whether valid. For example field value or field type.
        let schema = &self.schema;
        let record = schema.verify_record(record)?;
        let page = self
            .page_manager
            .get_page_with_space(schema.estimated_size());

        // Find the first empty slot in the bitmap.
        // entry number of the first free slot and store it in entry number;
        // and(2) we count the total number of entries on this page.

        todo!()
    }

    /// Retrieves a record from the table, throwing an exception if no such
    /// record exists.
    pub fn search(&self, id: RecordId) -> Result<&Record> {
        todo!()
    }

    /// Updates an existing record with new values and returns the existing
    /// record. stats is updated accordingly. An exception is thrown if
    /// recordId does not correspond to and existing record in the table.
    pub fn update(&self, old_record_id: RecordId, updated: Record) -> Result<&Record> {
        todo!()
    }

    /// Deletes and returns the record specified bu recordId from the table and
    /// updates stats, freePageNums and numRecords as necessary. An
    /// exception is thrown if recordId dose not correspond to an existing
    /// record in the table.
    pub fn delete(&self, id: RecordId) -> Result<Record> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use bitmaps::Bitmap;

    #[test]
    fn test_bitmap() {
        let mut bitmap = Bitmap::<10>::new();
        assert_eq!(bitmap.set(5, true), false);
        assert_eq!(bitmap.set(5, true), true);
        assert_eq!(bitmap.get(5), true);
        assert_eq!(bitmap.get(6), false);
        assert_eq!(bitmap.len(), 1);
        assert_eq!(bitmap.set(3, true), false);
        assert_eq!(bitmap.len(), 2);
        assert_eq!(bitmap.first_index(), Some(3));
        assert_eq!(bitmap.last_index(), Some(5));
    }
}
