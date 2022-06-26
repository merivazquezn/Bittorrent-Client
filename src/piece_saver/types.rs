use super::sender::types::PieceSaverSender;
use super::worker::types::PieceSaverWorker;
use crate::piece_manager::sender::PieceManagerSender;
use std::sync::mpsc;

#[derive(Debug)]
pub enum PieceSaverMessage {
    ValidateAndSavePiece(u32, Vec<u8>),
    StopSaving,
}

pub fn new_piece_saver(
    piece_manager_sender: PieceManagerSender,
    sha1_pieces: Vec<Vec<u8>>,
    download_path: String,
) -> (PieceSaverSender, PieceSaverWorker) {
    let (tx, rx) = mpsc::channel();

    (
        PieceSaverSender { sender: tx },
        PieceSaverWorker {
            receiver: rx,
            piece_manager_sender,
            sha1_pieces,
            download_path,
        },
    )
}
