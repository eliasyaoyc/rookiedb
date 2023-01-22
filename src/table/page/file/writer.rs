pub struct PageWriter {}

impl PageWriter {
    pub fn new() -> PageWriter {
        Self {}
    }

    pub async fn flush() -> std::io::Result<()> {
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
