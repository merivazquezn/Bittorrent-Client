use crate::download_manager::save_piece_in_disk;
use crate::download_manager::Piece;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::types::PieceSaverMessage;
use sha1::{Digest, Sha1};
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

pub struct PieceSaverWorker {
    pub receiver: Receiver<PieceSaverMessage>,
    pub piece_manager_sender: PieceManagerSender,
    pub sha1_pieces: Vec<Vec<u8>>,
    pub download_path: String,
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
                piece_number: 0,
                data: piece_bytes,
            };
            save_piece_in_disk(&piece, &self.download_path).unwrap();
            // Msg not yet implemented by logger
            // logger.log_piece(piece_index);
            // piece_manager_sender.successfull_piece_download(piece_index);
        } else {
            // Err(PeerConnectionError::PieceRequestingError(
            //     "Invalid piece received".to_string(),
            // ));
            // piece_manager_sender.failed_piece_download(piece_index);
        }
    }

    pub fn listen(&self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            match message {
                PieceSaverMessage::StopSaving => {
                    // Msg not yet implemented by logger
                    // logger.stop_logging();
                    break;
                }
                PieceSaverMessage::ValidateAndSavePiece(piece_index, piece_bytes) => {
                    self.make_validation_and_save_piece(piece_index, piece_bytes);
                }
            }
        }

        Ok(())
    }
}
