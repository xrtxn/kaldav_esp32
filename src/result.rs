use std::convert::Into;

pub type Result<T = ()> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Misc(String),
    #[error("Parser error: {0}")]
    Parser(#[from] ical::parser::ParserError),
    #[error("HTTP error: {0}")]
    Http(#[from] attohttpc::Error),
}

impl Error {
    pub fn new<S>(message: S) -> Self
    where
        S: Into<String>,
    {
        Self::Misc(message.into())
    }
}
