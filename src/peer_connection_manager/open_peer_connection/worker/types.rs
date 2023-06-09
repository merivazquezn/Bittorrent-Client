use super::super::types::OpenPeerConnectionMessage;
use crate::constants::*;
use crate::logger::CustomLogger;
use crate::peer::*;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use log::*;
use std::sync::mpsc::Receiver;
const MIN_FAILED_CONNECTIONS: u32 = 1;
const LOGGER: CustomLogger = CustomLogger::init("Open Peer Connection");
use crate::ui::PeerStatistics;
pub struct OpenPeerConnectionWorker {
    pub receiver: Receiver<OpenPeerConnectionMessage>,
    pub connection: PeerConnection,
    pub piece_manager_sender: PieceManagerSender,
    pub piece_saver_sender: PieceSaverSender,
    pub peer_connection_manager_sender: PeerConnectionManagerSender,
    pub failed_download_in_a_row: u32,
    pub is_open: bool,
}

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

        LOGGER.info(format!(
            "Piece {} received, sending it to piece saver",
            piece_index
        ));
        self.piece_saver_sender.validate_and_save_piece(
            piece_index,
            self.connection.get_peer_id(),
            piece_data,
        );

        Ok(())
    }

    pub fn listen(&mut self) -> Result<(), (String, Vec<u8>)> {
        self.connection.ui_message_sender.send_new_connection();
        let peer_statistics = PeerStatistics {
            torrentname: self.connection.metainfo.info.name.clone(),
            peerid: self.connection.peer_id.clone(),
            ip: self.connection.peer.ip.clone(),
            port: self.connection.peer.port,
            uploadrate: 0,
            downloadrate: 0,
            state: PeerConnectionState {
                client: PeerState {
                    chocked: self.connection.peer_choking,
                    interested: self.connection._am_interested,
                },
                peer: PeerState {
                    chocked: self.connection._am_choking,
                    interested: self.connection._peer_interested,
                },
            },
        };
        self.connection
            .ui_message_sender
            .send_peer_statistics(peer_statistics);
        loop {
            let message = self.receiver.recv().map_err(|_| {
                self.connection
                    .ui_message_sender
                    .send_closed_connection(self.connection.get_peer_id());
                self.piece_manager_sender
                    .failed_connection(self.connection.get_peer_id());
                self.peer_connection_manager_sender
                    .failed_connection(self.connection.get_peer_id());
                (
                    "Error trying to receive message from OpenPeerConnectionWorker".to_string(),
                    self.connection.get_peer_id().to_vec(),
                )
            })?;

            trace!(
                "peer connection worker with ip: {:?} received message: {:?}",
                self.connection.get_peer_ip(),
                message
            );
            match message {
                OpenPeerConnectionMessage::SendBitfield => self.send_bitfield(),
                OpenPeerConnectionMessage::DownloadPiece(piece_index) => {
                    if self.download_piece(piece_index).is_err() {
                        self.piece_manager_sender
                            .failed_download(piece_index, self.connection.get_peer_id());
                        self.failed_download_in_a_row += MIN_FAILED_CONNECTIONS;
                        if self.failed_download_in_a_row == MIN_FAILED_CONNECTIONS {
                            self.is_open = false;
                            trace!(
                                "Closing peer connection: {:?} after {:?} failed downloads in a row",
                                self.connection.get_peer_ip(),
                                MIN_FAILED_CONNECTIONS
                            );
                            self.connection
                                .ui_message_sender
                                .send_closed_connection(self.connection.get_peer_id());
                            self.peer_connection_manager_sender
                                .failed_connection(self.connection.get_peer_id());
                            // loop through all messages queued and call failed download for all of them, so they don't get lost in the void
                            self.receiver.try_iter().for_each(|message| {
                                if let OpenPeerConnectionMessage::DownloadPiece(piece_index) =
                                    message
                                {
                                    self.piece_manager_sender.failed_download(
                                        piece_index,
                                        self.connection.get_peer_id(),
                                    );
                                }
                            });

                            return Err((
                                format!(
                                    "Failed peer connection {:?}",
                                    self.connection.get_peer_id()
                                ),
                                self.connection.get_peer_id(),
                            ));
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
