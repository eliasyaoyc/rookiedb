use thiserror::Error;

#[derive(Error, Debug)]
pub enum PageError {
    #[error("invalid page expected {0}, but got {1}")]
    InvalidPage(String, String),
}
