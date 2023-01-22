use std::path::Path;

use super::{manifest::Manifest, Table};
use crate::error::Result;

impl Table {
    pub(crate) async fn recover<P: AsRef<Path>>(path: P) -> Result<()> {
        let mut _manifest = Manifest::open(path.as_ref()).await?;

        Ok(())
    }

    pub(crate) async fn apply() -> Result<()> {
        Ok(())
    }
}
