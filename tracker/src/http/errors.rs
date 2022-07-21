use std::io;

#[derive(Debug)]
pub enum HttpError {
    HttpError(String),
    IoError(io::Error),
    InvalidRequest(String),
}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> HttpError {
        HttpError::IoError(error)
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
        }
    }
}
