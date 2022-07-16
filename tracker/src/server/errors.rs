use bittorrent_rustico::server::ThreadPoolError;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum TrackerError {
    TcpError(io::Error),
    CreationError(String),
    ThreadPoolError(ThreadPoolError),
    JoinError,
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

impl fmt::Display for TrackerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TrackerError::TcpError(error) => write!(f, "Tcp error: {}", error),
            TrackerError::CreationError(error) => write!(f, "Creation error: {}", error),
            TrackerError::ThreadPoolError(error) => write!(f, "Thread pool error: {}", error),
            TrackerError::JoinError => write!(f, "Error trying to join acceptor thread"),
        }
    }
}
