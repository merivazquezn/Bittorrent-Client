use super::super::types::OpenPeerConnectionMessage;
use crate::peer::*;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use log::*;
use std::sync::mpsc::{Receiver, RecvError};

pub struct OpenPeerConnectionWorker {
    pub receiver: Receiver<OpenPeerConnectionMessage>,
    pub connection: PeerConnection,
    pub piece_manager_sender: PieceManagerSender,
    pub piece_saver_sender: PieceSaverSender,
}

#[allow(dead_code)]
impl OpenPeerConnectionWorker {
    fn send_bitfield(&self) {
        self.piece_manager_sender.peer_pieces(
            self.connection.get_peer_id(),
            self.connection.get_bitfield(),
        );
    }

    fn download_piece(&mut self, piece_index: u32) {
        const BLOCK_SIZE: u32 = 16 * u32::pow(2, 10);
        let piece_data: Vec<u8> = self
            .connection
            .request_piece(piece_index, BLOCK_SIZE)
            .map_err(|_| {
                PeerConnectionError::PieceRequestingError(
                    "Error trying to request piece".to_string(),
                )
            })
            .unwrap();

        self.piece_saver_sender
            .validate_and_save_piece(piece_index, piece_data);
    }

    pub fn listen(&mut self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            trace!(
                "Open peer connection with id: {:?} received message: {:?}",
                self.connection.get_peer_id(),
                message
            );
            match message {
                OpenPeerConnectionMessage::SendBitfield => self.send_bitfield(),
                OpenPeerConnectionMessage::DownloadPiece(piece_index) => {
                    self.download_piece(piece_index)
                }
                OpenPeerConnectionMessage::CloseConnection => break,
            }
        }
        Ok(())
    }
}
