use std::convert::{ Into, From };

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new<S>(message: S) -> Self where S: Into<String> {
        Error {
            message: message.into(),
        }
    }
}

impl From<::hyper::Error> for Error {
    fn from(err: ::hyper::Error) -> Error {
        Error {
            message: format!("{}", err),
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(err: ::std::io::Error) -> Error {
        Error {
            message: format!("{}", err),
        }
    }
}
