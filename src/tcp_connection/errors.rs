use std::fmt::Display;

#[derive(Debug)]
pub enum TlsHttpHandshakeError {
    HandshakeFailure,
    BlockingError,
}

#[derive(Debug)]
/// The error type that is returned when connecting to the tracker
pub enum TlsHttpError {
    /// Couldn't establish a connection to the tracker, failed in the handshake step
    InitialConnectionFailure(TlsHttpHandshakeError),

    /// Communication with the tracker failed once established
    CommunicationError(native_tls::Error),
}

impl From<native_tls::Error> for TlsHttpError {
    fn from(error: native_tls::Error) -> Self {
        TlsHttpError::CommunicationError(error)
    }
}

impl From<native_tls::HandshakeError<std::net::TcpStream>> for TlsHttpError {
    fn from(error: native_tls::HandshakeError<std::net::TcpStream>) -> Self {
        match error {
            native_tls::HandshakeError::Failure(_) => {
                TlsHttpError::InitialConnectionFailure(TlsHttpHandshakeError::HandshakeFailure)
            }
            native_tls::HandshakeError::WouldBlock(_) => {
                TlsHttpError::InitialConnectionFailure(TlsHttpHandshakeError::BlockingError)
            }
        }
    }
}

impl Display for TlsHttpHandshakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsHttpHandshakeError::HandshakeFailure => write!(f, "Handshake failure"),
            TlsHttpHandshakeError::BlockingError => write!(f, "Blocking error"),
        }
    }
}

impl Display for TlsHttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TlsHttpError::InitialConnectionFailure(err) => {
                write!(f, "InitialConnectionFailure: {}", err)
            }
            TlsHttpError::CommunicationError(err) => write!(f, "CommunicationError: {}", err),
        }
    }
}
