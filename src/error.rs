use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("corrupted {0}.")]
    Corrupted(String),
    #[error("invalid {0}.")]
    Invalid(String),
    #[error("too large size.")]
    TooLargeSize,
    #[error("{0} not found.")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
