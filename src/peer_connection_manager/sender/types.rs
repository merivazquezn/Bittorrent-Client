use crate::peer_connection_manager::types::PeerConnectionManagerMessage;
use std::sync::mpsc::Sender;
#[derive(Clone)]
pub struct PeerConnectionManagerSender {
    pub sender: Sender<PeerConnectionManagerMessage>,
}

#[allow(dead_code)]
impl PeerConnectionManagerSender {
    pub fn close_connections(&self) {
        let _ = self
            .sender
            .send(PeerConnectionManagerMessage::CloseConnections);
    }

    pub fn download_piece(&self, peer_id: Vec<u8>, piece_index: u32) {
        let _ = self
            .sender
            .send(PeerConnectionManagerMessage::DownloadPiece(
                peer_id,
                piece_index,
            ));
    }
}
