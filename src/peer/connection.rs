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
    bitfield: Bitfield,
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
            bitfield: Bitfield::new(),
        }
    }

    // function that converts a slice of bytes into a u32 be
    fn vec_be_to_u32(&self, bytes: &[u8]) -> u32 {
        let mut num = 0;
        for (i, byte) in bytes.iter().enumerate().take(4) {
            num += (*byte as u32) << (8 * i);
        }
        num
    }

    fn wait_for_message(&mut self) -> Result<PeerMessage, Box<dyn std::error::Error>> {
        let message = self.message_service.wait_for_message()?;
        match message.id {
            PeerMessageId::Unchoke => {
                self.peer_choking = false;
            }
            PeerMessageId::Bitfield => {
                self.bitfield.set_bitfield(&message.payload);
            }
            PeerMessageId::Piece => {
                let _piece_index = self.vec_be_to_u32(&message.payload[0..=4]);
                let _offset = self.vec_be_to_u32(&message.payload[4..=8]);
                let block = message.payload[8..].to_vec();
                debug!("block received length: {}", block.len());
            }
            _ => {
                return Err("unhandled message".into());
            }
        }
        Ok(message)
    }

    fn wait_until_ready(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.wait_for_message()?;
            if !self.peer_choking && self.bitfield.non_empty() {
                break;
            }
        }
        Ok(())
    }

    fn request_pieces(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.message_service
            .send_message(&PeerMessage::request(0, 0, 16 * u32::pow(2, 10)))?;

        loop {
            let message = self.wait_for_message()?;
            if message.id == PeerMessageId::Piece {
                break;
            }
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.message_service
            .handshake(&self.metainfo.info_hash, &self.client_peer_id)?;
        self.message_service.send_message(&PeerMessage::unchoke())?;
        self.message_service
            .send_message(&PeerMessage::interested())?;
        self.wait_until_ready()?;
        self.request_pieces()?;
        Ok(())
    }
}

#[cfg(test)]

mod tests {
    //use super::*;

    // #[test]
    // fn
}
