use crate::bencode::BencodeDecoderError;
use crate::tcp_connection;
use std::fmt::Display;
use std::fmt::Formatter;

/// The error type that is returned when connecting to the tracker
pub enum TrackerError {
    TcpConnectionError(tcp_connection::TcpConnectionError),

    /// The Bencode decoder failed to decode the response from the tracker
    BencodingError(BencodeDecoderError),

    /// Couldn't read or write message from socket
    IoError(std::io::Error),

    ResponseError(String),
    /// The tracker response was invalid
    InvalidResponse,
}

impl From<std::io::Error> for TrackerError {
    fn from(error: std::io::Error) -> Self {
        TrackerError::IoError(error)
    }
}

impl From<BencodeDecoderError> for TrackerError {
    fn from(error: BencodeDecoderError) -> Self {
        TrackerError::BencodingError(error)
    }
}

// Implement From <Box<dyn std::error::Error + Send + Sync>> for tracker::errors::TrackerError
impl<T: Display + Send + Sync + 'static> From<Box<T>> for TrackerError {
    fn from(error: Box<T>) -> Self {
        TrackerError::ResponseError(format!("{}", error))
    }
}

// implement from TcpConnectionError for TrackerError
impl From<tcp_connection::TcpConnectionError> for TrackerError {
    fn from(error: tcp_connection::TcpConnectionError) -> Self {
        TrackerError::TcpConnectionError(error)
    }
}

impl Display for TrackerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackerError::BencodingError(error) => {
                write!(f, "Failed bencode-decoded tracker response: {}", error)
            }
            TrackerError::IoError(error) => write!(f, "Failed to read/write data: {}", error),
            TrackerError::InvalidResponse => write!(f, "Tracker response is invalid"),
            TrackerError::ResponseError(err) => write!(f, "Tracker response error: {}", err),
            TrackerError::TcpConnectionError(error) => {
                write!(f, "Failed to use tcp connection: {}", error)
            }
        }
    }
}
