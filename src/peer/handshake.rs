use super::PeerConnectionError;

pub trait IHandshakeService {
    fn handshake(&mut self, info_hash: &[u8], peer_id: &[u8]) -> Result<(), PeerConnectionError>;
}
