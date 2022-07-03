use std::fmt;

use crate::logger::LoggerError;

#[derive(Debug)]
pub enum PeerConnectionError {
    LoggerCreationFailure(LoggerError),
    IoError(std::io::Error),
    PeerMessageError(IPeerMessageServiceError),
    PieceRequestingError(String),
    InitialConnectionError(String),
    PieceSavingError(String),
    LoggingPieceError(String),
    JoiningError(String),
}

#[derive(Debug)]
pub enum IPeerMessageServiceError {
    PeerHandshakeError(String),
    SendingMessageError(String),
    ReceivingMessageError(String),
    InvalidResponse(String),
    UnhandledMessage,
    InvalidMessageId,
    IOError(std::io::Error),
}

impl From<std::io::Error> for IPeerMessageServiceError {
    fn from(error: std::io::Error) -> Self {
        IPeerMessageServiceError::IOError(error)
    }
}

impl From<LoggerError> for PeerConnectionError {
    fn from(error: LoggerError) -> Self {
        PeerConnectionError::LoggerCreationFailure(error)
    }
}

impl From<std::io::Error> for PeerConnectionError {
    fn from(error: std::io::Error) -> Self {
        PeerConnectionError::IoError(error)
    }
}

impl From<IPeerMessageServiceError> for PeerConnectionError {
    fn from(error: IPeerMessageServiceError) -> Self {
        PeerConnectionError::PeerMessageError(error)
    }
}

impl fmt::Display for IPeerMessageServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IPeerMessageServiceError::PeerHandshakeError(reason) => write!(f, "{}", reason),
            IPeerMessageServiceError::SendingMessageError(reason) => write!(f, "{}", reason),
            IPeerMessageServiceError::InvalidResponse(reason) => write!(f, "{}", reason),
            IPeerMessageServiceError::ReceivingMessageError(reason) => {
                write!(f, "Receiving message error: {}", reason)
            }
            IPeerMessageServiceError::UnhandledMessage => write!(
                f,
                "Peer received a message which does not know how to handle"
            ),
            IPeerMessageServiceError::InvalidMessageId => {
                write!(f, "Received message id which is not valid")
            }
            IPeerMessageServiceError::IOError(error) => {
                write!(f, "IO Error when flushing: {}", error)
            }
        }
    }
}

impl fmt::Display for PeerConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PeerConnectionError::LoggerCreationFailure(error) => {
                write!(f, "Logger creation failure: {}", error)
            }
            PeerConnectionError::IoError(error) => {
                write!(f, "IO Error: {}", error)
            }
            PeerConnectionError::PeerMessageError(error) => {
                write!(f, "Peer message error: {}", error)
            }
            PeerConnectionError::PieceRequestingError(error) => {
                write!(f, "Piece requesting error: {}", error)
            }
            PeerConnectionError::InitialConnectionError(error) => {
                write!(f, "Initial connection error: {}", error)
            }
            PeerConnectionError::PieceSavingError(error) => {
                write!(f, "Piece saving error: {}", error)
            }
            PeerConnectionError::LoggingPieceError(error) => {
                write!(f, "Logging piece error: {}", error)
            }
            PeerConnectionError::JoiningError(error) => {
                write!(f, "Joining error: {}", error)
            }
        }
    }
}
