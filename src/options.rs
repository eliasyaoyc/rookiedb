use std::fmt::Debug;

pub struct Options {
    pub path: String,
    /// Represent system how to hande `NULL` value.
    /// - null_equal: represent all null value is equals (Default setting.).
    /// - null_unequal: represent all null value is independent.
    /// - null_ignore: represent ignore null value.
    pub stats_null_method: String,
    /// Represent the data of statistic whether persistent to disk.
    pub stats_persistent: bool,
    pub num_records_per_page: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            path: "".to_owned(),
            num_records_per_page: 8,
            stats_null_method: "nulls_equal".to_owned(),
            stats_persistent: true,
        }
    }
}

impl Debug for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Options").finish()
    }
}
