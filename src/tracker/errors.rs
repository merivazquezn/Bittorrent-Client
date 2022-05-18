use crate::bencode::BencodeDecoderError;
use native_tls;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum TrackerHandshakeError {
    HandshakeFailure,
    BlockingError,
}

#[derive(Debug)]
/// The error type that is returned when connecting to the tracker
pub enum TrackerError {
    /// Couldn't establish a connection to the tracker, failed in the handshake step
    InitialConnectionFailure(TrackerHandshakeError),

    /// Communication with the tracker failed once established
    CommunicationError(native_tls::Error),

    /// The Bencode decoder failed to decode the response from the tracker
    BencodingError(BencodeDecoderError),

    /// Couldn't read or write message from socket
    IoError(std::io::Error),

    /// The tracker response was invalid
    InvalidResponse,
}

impl From<native_tls::Error> for TrackerError {
    fn from(error: native_tls::Error) -> Self {
        TrackerError::CommunicationError(error)
    }
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

impl From<native_tls::HandshakeError<std::net::TcpStream>> for TrackerError {
    fn from(error: native_tls::HandshakeError<std::net::TcpStream>) -> Self {
        match error {
            native_tls::HandshakeError::Failure(_) => {
                TrackerError::InitialConnectionFailure(TrackerHandshakeError::HandshakeFailure)
            }
            native_tls::HandshakeError::WouldBlock(_) => {
                TrackerError::InitialConnectionFailure(TrackerHandshakeError::BlockingError)
            }
        }
    }
}

impl Display for TrackerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackerError::CommunicationError(err) => {
                write!(f, "Falied to send/receive data from tracker: {}", err)
            }
            TrackerError::BencodingError(error) => {
                write!(f, "Failed parsing tracker answer: {}", error)
            }
            TrackerError::IoError(error) => write!(f, "Failed to read/write data: {}", error),
            TrackerError::InitialConnectionFailure(err) => match err {
                TrackerHandshakeError::HandshakeFailure => {
                    write!(f, "Failed to handshake with tracker")
                }
                TrackerHandshakeError::BlockingError => {
                    write!(f, "I/O operations with tracker are blocked")
                }
            },
            TrackerError::InvalidResponse => write!(f, "Tracker response is invalid"),
        }
    }
}
