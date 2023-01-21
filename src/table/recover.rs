use std::path::Path;

use super::manifest::Manifest;
use crate::error::Result;

pub(crate) async fn recover<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut manifest = Manifest::open(path.as_ref()).await?;
    
    Ok(())
}

pub(crate) async fn apply() -> Result<()> {
    Ok(())
}
