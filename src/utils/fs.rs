use std::path::Path;

use async_fs::File;
use futures::TryStreamExt;

use crate::error::Result;

pub(crate) async fn open<P: AsRef<Path>>(path: P) -> std::io::Result<File> {
    async_fs::OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(true)
        .open(path)
        .await
}

pub(crate) async fn rename<P: AsRef<Path>, Q: AsRef<Path>>(src: P, dst: Q) -> Result<()> {
    Ok(async_fs::rename(src, dst).await?)
}

pub(crate) async fn remove_file<P: AsRef<Path>>(path: P) -> Result<()> {
    Ok(async_fs::remove_file(path).await?)
}

pub(crate) async fn remove_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    Ok(async_fs::remove_dir(path).await?)
}

pub(crate) async fn create_file<P: AsRef<Path>>(path: P) -> Result<File> {
    Ok(File::create(path).await?)
}

pub(crate) async fn create_dir<P: AsRef<Path>>(path: P) -> Result<()> {
    Ok(async_fs::create_dir_all(path).await?)
}

pub(crate) async fn read_dir<P: AsRef<Path>>(path: P) -> Result<Vec<String>> {
    let mut entries = async_fs::read_dir(path).await?;
    let mut files = Vec::<String>::new();
    while let Some(entry) = entries.try_next().await? {
        files.push(entry.file_name().to_string_lossy().to_string());
    }
    Ok(files)
}

pub(crate) async fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>> {
    Ok(async_fs::read(path).await?)
}
