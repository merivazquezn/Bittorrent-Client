use crate::logger::LoggerError;
use crate::peer::IPeerMessageServiceError;
use std::fmt;

#[derive(Debug)]
/// Error type for the server and its connections with other peers
pub enum ServerError {
    /// The Tcp connection failed to read or write data
    TcpStreamError(std::io::Error),

    /// A thread couldn0t be correctly joined
    JoinError,

    /// The threadpool failed, saves the underlying ThreadPoolError with its underlying cause
    ThreadPoolError(ThreadPoolError),

    /// There was an error in the received request message from the other peer
    PieceRequestError(String),

    /// The logger couldn't be created, saves the LoggerError with its underlying cause
    LoggerCreationError(LoggerError),
}

#[derive(Debug)]
/// Error type for the threadpool
pub enum ThreadPoolError {
    /// There was a problem creating the threadpool
    /// Stores a String explaining the error
    CreationError(String),
}

impl From<std::io::Error> for ServerError {
    fn from(error: std::io::Error) -> Self {
        ServerError::TcpStreamError(error)
    }
}

impl From<ThreadPoolError> for ServerError {
    fn from(error: ThreadPoolError) -> Self {
        ServerError::ThreadPoolError(error)
    }
}

impl From<IPeerMessageServiceError> for ServerError {
    fn from(error: IPeerMessageServiceError) -> Self {
        ServerError::PieceRequestError(error.to_string())
    }
}

impl From<LoggerError> for ServerError {
    fn from(error: LoggerError) -> Self {
        ServerError::LoggerCreationError(error)
    }
}

impl fmt::Display for ThreadPoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ThreadPoolError::CreationError(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::TcpStreamError(error) => write!(f, "TcpStream error: {}", error),
            ServerError::JoinError => write!(f, "Error trying to join thread"),
            ServerError::ThreadPoolError(error) => write!(f, "ThreadPool error: {}", error),
            ServerError::PieceRequestError(reason) => write!(f, "Piece request error: {}", reason),
            ServerError::LoggerCreationError(error) => {
                write!(f, "Logger creation error: {}", error)
            }
        }
    }
}
