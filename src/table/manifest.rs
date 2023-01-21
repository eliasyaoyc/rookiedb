use std::path::Path;

use crate::error::Result;

pub struct Manifest {}

impl Manifest {
    pub(crate) async fn open(path: &Path) -> Result<Manifest> {
        todo!()
    }
}
