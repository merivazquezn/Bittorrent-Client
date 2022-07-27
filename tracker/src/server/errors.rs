use bittorrent_rustico::server::ThreadPoolError;
use std::fmt;
use std::io;

use crate::http::HttpError;

#[derive(Debug)]
pub enum TrackerError {
    TcpError(io::Error),
    CreationError(String),
    ThreadPoolError(ThreadPoolError),
    JoinError,
    InvalidEndpoint(String),
    HttpError(HttpError),
    AnnounceError(AnnounceError),
}

#[derive(Debug)]
pub enum AnnounceError {
    MissingParam(String),
    BadRequest,
}

impl From<HttpError> for TrackerError {
    fn from(error: HttpError) -> Self {
        TrackerError::HttpError(error)
    }
}

impl From<ThreadPoolError> for TrackerError {
    fn from(error: ThreadPoolError) -> TrackerError {
        TrackerError::ThreadPoolError(error)
    }
}

impl From<io::Error> for TrackerError {
    fn from(error: io::Error) -> Self {
        TrackerError::TcpError(error)
    }
}

impl From<AnnounceError> for TrackerError {
    fn from(error: AnnounceError) -> Self {
        TrackerError::AnnounceError(error)
    }
}

impl fmt::Display for AnnounceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnnounceError::MissingParam(param) => {
                write!(f, "Mandatory parameter is missing: {}", param)
            }
            AnnounceError::BadRequest => {
                write!(f, "Bad request")
            }
        }
    }
}

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TrackerError::TcpError(error) => write!(f, "Tcp error: {}", error),
            TrackerError::HttpError(error) => write!(f, "Http error; {}", error),
            TrackerError::CreationError(error) => write!(f, "Creation error: {}", error),
            TrackerError::ThreadPoolError(error) => write!(f, "Thread pool error: {}", error),
            TrackerError::JoinError => write!(f, "Error trying to join acceptor thread"),
            TrackerError::AnnounceError(error) => write!(f, "Announce error: {}", error),
            TrackerError::InvalidEndpoint(endpoint) => {
                write!(f, "Received request on invalid endpoint: {}", endpoint)
            }
        }
    }
}
