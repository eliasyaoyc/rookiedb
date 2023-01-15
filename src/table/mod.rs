mod page;
mod record;
mod schema;
mod stats;

use std::fmt::Debug;

use anyhow::Result;

use self::page::PageDirectory;
use self::schema::Schema;
use self::stats::TableStats;

/// A Table represents a database table with which users can insert, get,
/// update and delete records.
///
/// # Persistence
/// Every table is persisted in its own `page_directory`, which interfaces with
/// buffer and disk to save it to disk.
///
/// A table can be loaded again by simply constructing it with the same parameters.
///
/// # Storage Format
/// All pages are data pages - there are no header pages, because all metadata is
/// stored elsewhere (as rows in the _metadata.tables table). Every daa
/// which records inthe page are valid. The values of n and m are set to maximize the
/// number of records per page.
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
/// - The first page (Page 0) is a data page. The first byte of this data page is a bitmap, and the
///   next four bytes are each records. The first and fourth bytes are set indicating that record 0
///   and record 3 ar valid. Record 1 an record 2 ar invalid, so we ignore their contents.
///   Similarly, the last four bits of the bitmap are unused, so we ignore their contents.
/// - The second and third page (Page 1 and 2) are also data pages and are formatted similar to Page
///   0.
///
/// When we add a record to a table, we add it to the very first free slot in the table.
///
/// Some tables have large records. In order to efficiently handle tables with large records
/// (that still fit on a page), we format these tables a bit differently,
/// by giving each record a full page. Tables with full page records do not have a bitmap.
/// Instead, each allocated page is a single record, and we indicate that a page dose
/// not contain a record by simply freeing the page.
///
/// In some cases, this behavior may be desriable even for small records(our database only
/// supports locking at the page level, so in cases where tuple-level locks are necessary even
/// at the cost of an I/O per tuple, a full page record may be desirable),
/// and may be explicitly toggled on with the `set_full_page_records` methods.
pub struct Table {
    /// The name of the table.
    table_name: String,
    /// The schema of the table.
    schema: Schema,
    /// The page directory of the table.
    page_dir: PageDirectory,
    /// The size of the bitmap found at the beginning of each data page.
    bitmap_size: usize,
    /// The number of records on each data page.
    num_records_per_page: usize,
    /// Statistics about the contents of the database.
    stats: TableStats,
    // todo lock?
}

impl Debug for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table")
            .field("name", &self.table_name)
            .field("schema", &self.schema)
            .field("bitmap_size", &self.bitmap_size)
            .field("num_records_per_page", &self.num_records_per_page)
            .finish()
    }
}

impl Table {
    /// Create a new table.
    pub fn new(table_name: &str) -> Self {
        todo!()
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn num_records_per_page(&self) -> usize {
        self.num_records_per_page
    }

    // todo(consider): move to outer struct, e.g. database or other.
    pub fn statistics(&self) -> &TableStats {
        &self.stats
    }

    pub fn insert(&self) -> Result<()> {
        todo!()
    }
}
