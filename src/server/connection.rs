use crate::peer::IHandshakeService;
use crate::peer::PeerConnectionError;
use log::*;
use std::net::TcpStream;

#[allow(dead_code)]
pub struct ServerConnection;

pub struct HandshakeServerService;

impl IHandshakeService for HandshakeServerService {
    fn handshake(&mut self, _info_hash: &[u8], _peer_id: &[u8]) -> Result<(), PeerConnectionError> {
        Ok(())
    }
}

impl ServerConnection {
    pub fn run(mut _stream: TcpStream) {
        debug!("Started connection with client");
    }
}
