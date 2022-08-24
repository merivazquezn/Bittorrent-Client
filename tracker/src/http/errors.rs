use std::io;
use std::str::Utf8Error;
#[derive(Debug)]
pub enum HttpError {
    HttpError(String),
    IoError(io::Error),
    InvalidRequest(String),
    Utf8Error(Utf8Error),
    ParseIntError(core::num::ParseIntError)
}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> HttpError {
        HttpError::IoError(error)
    }
}

impl From<Utf8Error> for HttpError {
    fn from(error: Utf8Error) -> Self {
        HttpError::Utf8Error(error)
    }
}

impl From<core::num::ParseIntError> for HttpError {
    fn from(error: core::num::ParseIntError) -> HttpError {
        HttpError::ParseIntError(error)
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HttpError::HttpError(reason) => write!(f, "HttpError: {}", reason),
            HttpError::IoError(reason) => write!(f, "IoError: {}", reason),
            HttpError::InvalidRequest(request) => {
                write!(f, "Received Invalid Http Request: {}", request)
            }
            HttpError::Utf8Error(error) => write!(f, "Utf8Error: {}", error),
            HttpError::ParseIntError(error) => write!(f, "ParseIntError: {}", error)
        }
    }
}
