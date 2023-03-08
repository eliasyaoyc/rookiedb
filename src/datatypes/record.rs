pub struct RecordId(pub u64, pub usize);

pub fn new_record_id(page_num: u64, entry_num: usize) -> RecordId {
    RecordId(page_num, entry_num)
}

pub struct Record {}
