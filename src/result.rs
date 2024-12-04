use std::convert::Into;
use esp_idf_svc::io::EspIOError;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Misc(String),
    #[error("Parser error: {0}")]
    Parser(#[from] ikal::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] EspIOError),
}

impl Error {
    pub fn new<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Misc(message.into())
    }
}
