use std::fmt;

use crate::logger::LoggerError;

#[derive(Debug)]
pub enum PeerConnectionError {
    LoggerCreationFailure(LoggerError),
    IoError(std::io::Error),
    PeerMessageError(PeerMessageServiceError),
    PieceRequestingError(String),
}

#[derive(Debug)]
pub enum PeerMessageServiceError {
    PeerHandshakeError(String),
    SendingMessageError(String),
    ReceivingMessageError(String),
    InvalidResponse(String),
    UnhandledMessage,
    InvalidMessageId,
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

impl From<PeerMessageServiceError> for PeerConnectionError {
    fn from(error: PeerMessageServiceError) -> Self {
        PeerConnectionError::PeerMessageError(error)
    }
}

impl fmt::Display for PeerMessageServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PeerMessageServiceError::PeerHandshakeError(reason) => write!(f, "{}", reason),
            PeerMessageServiceError::SendingMessageError(reason) => write!(f, "{}", reason),
            PeerMessageServiceError::InvalidResponse(reason) => write!(f, "{}", reason),
            PeerMessageServiceError::ReceivingMessageError(reason) => write!(f, "{}", reason),
            PeerMessageServiceError::UnhandledMessage => write!(
                f,
                "Peer received a message which does not know how to handle"
            ),
            PeerMessageServiceError::InvalidMessageId => {
                write!(f, "Received message id which is not valid")
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
        }
    }
}
