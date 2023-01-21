use thiserror::Error;

use crate::page::error::PageError;

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

impl From<PageError> for Error {
    fn from(error: PageError) -> Self {
        todo!()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
