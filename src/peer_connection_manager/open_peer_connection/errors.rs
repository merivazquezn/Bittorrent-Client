use crate::peer::PeerConnectionError;
use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum OpenPeerConnectionError {
    PeerConnectionError(PeerConnectionError),
}

impl From<PeerConnectionError> for OpenPeerConnectionError {
    fn from(error: PeerConnectionError) -> Self {
        OpenPeerConnectionError::PeerConnectionError(error)
    }
}

impl Display for OpenPeerConnectionError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            OpenPeerConnectionError::PeerConnectionError(error) => {
                write!(f, "OpenPeerConnection Error: {}", error)
            }
        }
    }
}
