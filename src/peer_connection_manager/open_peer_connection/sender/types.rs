use super::super::types::OpenPeerConnectionMessage;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct OpenPeerConnectionSender {
    pub sender: Sender<OpenPeerConnectionMessage>,
}

impl OpenPeerConnectionSender {
    pub fn close_connection(&self) {
        let _ = self.sender.send(OpenPeerConnectionMessage::CloseConnection);
    }

    pub fn send_bitfield(&self) {
        let _ = self.sender.send(OpenPeerConnectionMessage::SendBitfield);
    }

    pub fn download_piece(&self, piece_index: u32) {
        let _ = self
            .sender
            .send(OpenPeerConnectionMessage::DownloadPiece(piece_index));
    }
}
