use crate::download_manager::save_piece_in_disk;
use crate::download_manager::Piece;
use crate::logger::{CustomLogger, Logger};
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::types::PieceSaverMessage;
use crate::ui::UIMessageSender;
use log::*;
use sha1::{Digest, Sha1};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

const LOGGER: CustomLogger = CustomLogger::init("Piece Saver");

pub struct PieceSaverWorker {
    pub receiver: Receiver<PieceSaverMessage>,
    pub piece_manager_sender: PieceManagerSender,
    pub sha1_pieces: Vec<Vec<u8>>,
    pub download_path: String,
    pub ui_message_sender: UIMessageSender,
}

impl PieceSaverWorker {
    fn sha1_of(&self, vec: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(vec);
        hasher.finalize().to_vec()
    }

    fn valid_piece(&self, piece_bytes: &[u8], piece_index: u32) -> bool {
        let real_piece_sha1 = self.sha1_pieces[piece_index as usize].to_vec();
        let recieved_piece_sha1 = self.sha1_of(piece_bytes);
        recieved_piece_sha1 == real_piece_sha1
    }

    fn make_validation_and_save_piece(&self, piece_index: u32, piece_bytes: Vec<u8>) -> bool {
        if !self.valid_piece(&piece_bytes, piece_index) {
            return false;
        }

        let piece = Piece {
            piece_number: piece_index,
            data: piece_bytes,
        };

        let download_path = format!("{}/pieces", String::from(&self.download_path));
        match save_piece_in_disk(&piece, &download_path) {
            Ok(()) => true,
            Err(_) => false,
        }
    }

    fn downloaded_piece_successfully(&self, piece_index: u32, peer_id: Vec<u8>, logger: &Logger) {
        self.piece_manager_sender
            .successful_download(piece_index, peer_id.clone());
        self.ui_message_sender.send_downloaded_piece(peer_id);
        LOGGER.info(format!("Piece {:^5} downloaded successfully", piece_index));
        let _ = logger.log_piece(piece_index);
    }

    pub fn listen(&self) -> Result<(), RecvError> {
        let (logger, handle) = Logger::new("./logs").unwrap();

        loop {
            let message = self.receiver.recv()?;

            match message {
                PieceSaverMessage::StopSaving => {
                    LOGGER.info_str("Stopping Piece Saver Worker");
                    break;
                }
                PieceSaverMessage::ValidateAndSavePiece(piece_index, peer_id, piece_bytes) => {
                    trace!("Piece saver received piece: {:?}", piece_index);
                    let successfuly_downloaded: bool =
                        self.make_validation_and_save_piece(piece_index, piece_bytes);

                    if successfuly_downloaded {
                        self.downloaded_piece_successfully(piece_index, peer_id, &logger);
                    } else {
                        self.piece_manager_sender
                            .failed_download(piece_index, peer_id);
                    }
                }
            }
        }

        logger.stop();
        let _ = handle.join();
        Ok(())
    }
}
