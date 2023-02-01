use std::path::Path;

use crate::error::Result;

pub enum ManifestEntry {
    BeginTxn,
    EndTnx,
    CommitTxn,

    BeginCheckpoint,
    EndCheckpoint,

    AllocPage,
    FreePage,

    AllocParition,
    FreePartition,

    UndoAllocPage,
    UndoFreePage,

    UndoAllocParition,
    UndoFreeParition,

    UpdatePage,
    UndoUpdatPage,
}

pub struct Manifest {}

impl Manifest {
    pub async fn open(_path: &Path) -> Result<Manifest> {
        todo!()
    }
}
