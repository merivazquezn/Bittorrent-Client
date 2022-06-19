use crate::logger::LoggerError;
use crate::peer::IPeerMessageServiceError;
use std::fmt;

#[derive(Debug)]
pub enum ServerError {
    TcpStreamError(std::io::Error),
    JoinError,
    ThreadPoolError(ThreadPoolError),
    PieceRequestError(String),
    LoggerCreationError(LoggerError),
}

#[derive(Debug)]
pub enum ThreadPoolError {
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
