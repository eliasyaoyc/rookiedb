use bitflags::bitflags;

bitflags! {
     pub struct Compression: u8 {
        const NONE = 1;
        const SNAPPY = 2;
        const ZSTD = 4;
     }
}