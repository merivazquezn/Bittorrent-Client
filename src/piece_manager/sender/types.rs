use crate::peer::Bitfield;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::piece_manager::types::PieceManagerMessage;
use std::sync::mpsc::Sender;

#[derive(Clone)]
#[allow(dead_code)]
pub struct PieceManagerSender {
    pub sender: Sender<PieceManagerMessage>,
}

impl PieceManagerSender {
    pub fn start(&self, peer_connection_manager: PeerConnectionManagerSender) {
        let _ = self
            .sender
            .send(PieceManagerMessage::Init(peer_connection_manager));
    }

    pub fn stop(&self) {
        let _ = self.sender.send(PieceManagerMessage::Stop);
    }

    pub fn peer_pieces(&self, peer_id: Vec<u8>, bitfield: Bitfield) {
        let _ = self
            .sender
            .send(PieceManagerMessage::PeerPieces(peer_id, bitfield));
    }

    pub fn successful_download(&self, piece_index: u32) {
        let _ = self
            .sender
            .send(PieceManagerMessage::SuccessfulDownload(piece_index));
    }

    pub fn failed_download(&self, piece_index: u32) {
        let _ = self
            .sender
            .send(PieceManagerMessage::FailedDownload(piece_index));
    }
}
