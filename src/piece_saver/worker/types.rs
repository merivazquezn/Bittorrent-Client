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

    fn valid_piece(&self, piece_bytes: Vec<u8>, piece_index: u32) -> bool {
        let real_piece_sha1 = self.sha1_pieces[piece_index as usize].to_vec();
        let recieved_piece_sha1 = self.sha1_of(piece_bytes.as_slice());
        recieved_piece_sha1 == real_piece_sha1
    }

    pub fn make_validation_and_save_piece(&self, piece_index: u32, piece_bytes: Vec<u8>) {
        if self.valid_piece(piece_bytes.clone(), piece_index) {
            let piece = Piece {
                piece_number: piece_index,
                data: piece_bytes,
            };
            save_piece_in_disk(&piece, &self.download_path).unwrap();
        }
    }

    pub fn listen(&self) -> Result<(), RecvError> {
        let (logger, handle) = Logger::new("./logs").unwrap();

        loop {
            let message = self.receiver.recv()?;
            match message {
                PieceSaverMessage::StopSaving => {
                    break;
                }
                PieceSaverMessage::ValidateAndSavePiece(piece_index, piece_bytes) => {
                    trace!("Piece saver received piece: {:?}", piece_index);
                    self.make_validation_and_save_piece(piece_index, piece_bytes);
                    self.ui_message_sender.send_downloaded_piece();
                    LOGGER.info(format!("Piece {} downloaded successfully", piece_index));
                    let _ = logger.log_piece(piece_index);
                }
            }
        }

        logger.stop();
        let _ = handle.join();
        Ok(())
    }
}
