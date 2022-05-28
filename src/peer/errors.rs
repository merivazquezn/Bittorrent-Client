use std::error;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct PeerConnectionError(pub String);

impl Error for PeerConnectionError {}

impl From<Box<dyn error::Error>> for PeerConnectionError {
    fn from(error: Box<dyn error::Error>) -> Self {
        PeerConnectionError(format!("{}", error))
    }
}

impl fmt::Display for PeerConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Peer Connection Error: {}", self.0)
    }
}
