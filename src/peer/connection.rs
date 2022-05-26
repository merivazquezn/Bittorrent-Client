use log::*;
// use all the modules config, peer, tracker, metainfo
use super::types::*;
use super::Peer;
use crate::metainfo::Metainfo;

pub struct PeerConnection {
    _am_choking: bool,
    _am_interested: bool,
    peer_choking: bool,
    _peer_interested: bool,
    message_service: Box<dyn PeerMessageService>,
    metainfo: Metainfo,
    client_peer_id: Vec<u8>,
}

impl PeerConnection {
    pub fn new(
        _peer: &Peer,
        client_peer_id: &[u8],
        metainfo: &Metainfo,
        message_service: Box<dyn PeerMessageService>,
    ) -> Self {
        Self {
            _am_choking: true,
            _am_interested: true,
            peer_choking: true,
            _peer_interested: false,
            client_peer_id: client_peer_id.to_vec(),
            metainfo: metainfo.clone(),
            message_service,
        }
    }

    fn listen_for_message(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let message = self.message_service.wait_for_message()?;
            match message.id {
                PeerMessageId::Unchoke => {
                    self.peer_choking = false;
                    break;
                }
                PeerMessageId::Bitfield => {}
                _ => {
                    debug!("Unknown message received");
                }
            }
        }
        debug!("finished peer connection");
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.message_service
            .handshake(&self.metainfo.info_hash, &self.client_peer_id)?;
        self.message_service.send_message(&PeerMessage::unchoke())?;
        self.message_service
            .send_message(&PeerMessage::interested())?;
        self.listen_for_message()?;
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    //use super::*;

    // #[test]
    // fn
}
