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

    pub fn failed_connection(&self, peer_id: Vec<u8>) {
        let _ = self
            .sender
            .send(PieceManagerMessage::FailedConnection(peer_id));
    }

    pub fn have(&self, peer_id: Vec<u8>, piece_index: u32) {
        let _ = self
            .sender
            .send(PieceManagerMessage::Have(peer_id, piece_index));
    }

    pub fn first_connections_started(&self) {
        let _ = self
            .sender
            .send(PieceManagerMessage::FirstConnectionsStarted());
    }

    pub fn finished_stablishing_connections(&self) {
        let _ = self
            .sender
            .send(PieceManagerMessage::FinishedStablishingConnections());
    }

    pub fn reasked_tracker(&self) {
        let _ = self.sender.send(PieceManagerMessage::ReaskedTracker());
    }
}
