use std::collections::HashMap;

use super::frame::Frame;

/// Reserve 36 bytes on each page for bookkeeing for recovery
/// (used to store the pageLSN, and to ensure that a redo-only/undo-only log record can
/// fit on one page).
const RESERVED_SPACE: usize = 36;

/// Implementation of a buffer manager, with configurable page replaceent policies.
/// Data is stored in page-size byte arrays, and returnd in a `Frame` object specific
/// to the page loaded (evicting and loading a new page into the frame will result in
/// a new `Frame` object, with the same underlying byte array). with old `Frame` objects
/// backed by the same byte array marked as invalid.
pub struct BufferManager {
    /// Buffer frames.
    frames: Vec<Frame>,

    /// Map of page number to frame index.
    page_to_frame: HashMap<u64, usize>,

    /// Index of the first free frame.
    first_free_index: usize,
}
