use super::super::types::OpenPeerConnectionMessage;
use crate::constants::*;
use crate::peer::*;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use log::*;
use std::sync::atomic::Ordering;
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
        // in a new thread, use self.connection.ui_message_sender to send download_rate to UI
        // the way download_rate is calculated, is by dividing the number rof self.connection.last_downloaded_pieces, which
        // is an atomicuint wrapped in an Arc
        // by the number of seconds since the last time the download_rate was calculated
        // the calculation is done every 5 seconds
        // we will join the thread when the connection is closed
        let (tx, rx) = std::sync::mpsc::channel();
        let ui_msg_sender = self.connection.ui_message_sender.clone();
        let last_downloaded_pieces = self.connection.last_downloaded_pieces.clone();
        let peer_conn_id = self.connection.get_peer_id();
        let rate_measure_handle = std::thread::spawn(move || {
            let mut time = std::time::Instant::now();
            loop {
                // print last_downloaded_pieces
                if rx.try_recv().is_ok() {
                    return;
                }
                if time.elapsed().as_secs() >= 5 {
                    let downloaded_pieces = last_downloaded_pieces.load(Ordering::Relaxed);
                    // donloaded_pieces / time elapsed as secods float
                    let download_rate = downloaded_pieces as f32 / time.elapsed().as_secs() as f32;
                    ui_msg_sender.send_download_rate(download_rate, &peer_conn_id);
                    time = std::time::Instant::now();
                }
                last_downloaded_pieces.store(0, Ordering::Relaxed);

                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });

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
                        self.piece_manager_sender
                            .failed_download(piece_index, self.connection.get_peer_id());
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
        tx.send(true).unwrap();
        rate_measure_handle.join().unwrap();
        trace!(
            "peer connection worker with ip: {:?} closed",
            self.connection.get_peer_ip()
        );
        Ok(())
    }
}
