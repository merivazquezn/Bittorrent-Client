use super::super::types::OpenPeerConnectionMessage;
use crate::constants::*;
use crate::peer::*;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use log::*;
use std::sync::mpsc::{Receiver, RecvError};

pub struct OpenPeerConnectionWorker {
    pub receiver: Receiver<OpenPeerConnectionMessage>,
    pub connection: PeerConnection,
    pub piece_manager_sender: PieceManagerSender,
    pub piece_saver_sender: PieceSaverSender,
    pub peer_connection_manager_sender: PeerConnectionManagerSender,
    pub failed_download_in_a_row: u32,
    pub is_open: bool,
}

#[allow(dead_code)]
impl OpenPeerConnectionWorker {
    fn send_bitfield(&self) {
        self.piece_manager_sender.peer_pieces(
            self.connection.get_peer_id(),
            self.connection.get_bitfield(),
        );
    }

    fn download_piece(&mut self, piece_index: u32) -> Result<(), PeerConnectionError> {
        let piece_data: Vec<u8> = self
            .connection
            .request_piece(
                piece_index,
                BLOCK_SIZE,
                self.connection.ui_message_sender.clone(),
            )
            .map_err(|_| {
                PeerConnectionError::PieceRequestingError(
                    "Error trying to request piece".to_string(),
                )
            })?;

        self.piece_saver_sender.validate_and_save_piece(
            piece_index,
            self.connection.get_peer_id(),
            piece_data,
        );

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            trace!(
                "peer connection worker with ip: {:?} received message: {:?}",
                self.connection.get_peer_ip(),
                message
            );
            match message {
                OpenPeerConnectionMessage::SendBitfield => self.send_bitfield(),
                OpenPeerConnectionMessage::DownloadPiece(piece_index) => {
                    if self.download_piece(piece_index).is_err() {
                        self.piece_manager_sender.failed_download(piece_index);
                        self.failed_download_in_a_row += 1;
                        if self.failed_download_in_a_row == 1 {
                            self.piece_manager_sender
                                .failed_connection(self.connection.get_peer_id());

                            // self.peer_connection_manager_sender
                            //     .failed_connection(self.connection.get_peer_id());
                            self.is_open = false;
                            // self.ui_message_sender.send_closes_conection();
                            error!("SE SACO A UN PEER DESDE OPNE PEER SE MANDO MSJ A PIECE MAN");
                            self.peer_connection_manager_sender
                                .failed_connection(self.connection.get_peer_id());

                            break;
                        }
                    } else {
                        self.failed_download_in_a_row = 0;
                    }
                }
                OpenPeerConnectionMessage::CloseConnection => break,
            }
        }
        trace!(
            "peer connection worker with ip: {:?} closed",
            self.connection.get_peer_ip()
        );
        Ok(())
    }
}
