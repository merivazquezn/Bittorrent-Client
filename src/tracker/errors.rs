use crate::bencode::BencodeDecoderError;
use crate::http::HttpsServiceError;
use std::fmt::Display;
use std::fmt::Formatter;
/// The error type that is returned when connecting to the tracker
#[derive(Debug)]
pub enum TrackerError {
    /// The Bencode decoder failed to decode the response from the tracker
    BencodeError(String),
    /// Http connection failed
    HttpError(String),
    /// The tracker response was invalid
    InvalidResponse(String),
}

impl From<BencodeDecoderError> for TrackerError {
    fn from(error: BencodeDecoderError) -> Self {
        TrackerError::BencodeError(error.to_string())
    }
}

// impl from HttpConnectionError for TrackerError
impl From<HttpsServiceError> for TrackerError {
    fn from(error: HttpsServiceError) -> Self {
        TrackerError::HttpError(error.to_string())
    }
}

impl Display for TrackerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TrackerError::InvalidResponse(error) => {
                write!(f, "Tracker response is invalid: {}", error)
            }
            TrackerError::HttpError(err) => write!(f, "Http error: {}", err),
            TrackerError::BencodeError(error) => write!(f, "Failed to parse bencode: {}", error),
        }
    }
}
