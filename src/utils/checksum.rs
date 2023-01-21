pub(crate) fn crc32(data: &[u8]) -> u32 {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

pub(crate) fn is_same(data: &[u8], crc: u32) -> bool {
    let mut hasher = crc32fast::Hasher::new();
    hasher.update(data);
    let new_crc = hasher.finalize();
    new_crc == crc
}
