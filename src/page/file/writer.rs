use async_fs::File;
use futures_lite::io::AsyncWriteExt;

pub struct PageWriter {}

impl PageWriter {
    pub fn new(capacity: usize) -> PageWriter {
        Self {}
    }

    pub async fn flush() -> std::io::Result<()> {
        let mut file = File::create("a.txt").await?;
        file.write_all(b"Hello, world!").await?;
        file.flush().await?;
        todo!()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_write() {}

    #[test]
    fn test_flush() {}
}
