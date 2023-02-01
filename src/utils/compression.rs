use bitflags::bitflags;

use crate::error::{Error, Result};

bitflags! {
     pub struct Compression: u8 {
        const NONE = 1;
        const SNAPPY = 2;
        const ZSTD = 4;
     }
}

pub fn compress<'a>(
    compression: Compression,
    raw: &'a [u8],
    compress_buf: &'a mut [u8],
) -> Result<&'a [u8]> {
    match compression {
        Compression::NONE => Ok(raw),
        Compression::SNAPPY => {
            let len = snap::raw::Encoder::new()
                .compress(raw, compress_buf)
                .map_err(|_| Error::Corrupted("snappy compression failed.".to_owned()))?;
            Ok(&compress_buf[..len])
        }
        Compression::ZSTD => {
            let raw_len = raw.len() as u64;
            compress_buf[0..std::mem::size_of::<u64>()].copy_from_slice(&raw_len.to_le_bytes());
            let dat_size = {
                let mut compressor = zstd::bulk::Compressor::new(3).unwrap();
                compressor
                    .compress_to_buffer(raw, &mut compress_buf[std::mem::size_of::<u64>()..])
                    .unwrap()
            } + std::mem::size_of::<u64>();
            Ok(&compress_buf[..dat_size])
        }
        _ => unreachable!(),
    }
}

pub fn decompress_into(
    compression: Compression,
    source: &[u8],
    target: &mut Vec<u8>,
) -> Result<()> {
    match compression {
        Compression::NONE => Ok(()),
        Compression::SNAPPY => {
            let len = snap::raw::Decoder::new()
                .decompress(source, target)
                .map_err(|_| Error::Corrupted("snappy decompression failed.".to_owned()))?;
            target.truncate(len);
            Ok(())
        }
        Compression::ZSTD => {
            let mut decompressor = zstd::bulk::Decompressor::new().unwrap();
            let len = decompressor
                .decompress_to_buffer(source, target)
                .map_err(|_| Error::Corrupted("zstd decompression failed.".to_owned()))?;
            target.truncate(len);
            Ok(())
        }
        _ => unreachable!(),
    }
}
