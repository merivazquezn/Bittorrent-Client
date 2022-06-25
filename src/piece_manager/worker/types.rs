use crate::peer::Bitfield;
use crate::piece_manager::types::PieceManagerMessage;
use crate::ui::UIMessageSender;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

pub struct PieceManagerWorker {
    pub reciever: Receiver<PieceManagerMessage>,
    pub bitfields: HashMap<Vec<u8>, Bitfield>,
    pub remaining_pieces: HashSet<u32>,
    pub pieces_downloading: HashSet<u32>,
    pub ui_message_sender: UIMessageSender,
}

impl PieceManagerWorker {
    fn piece_succesfully_downloaded(&mut self, piece_index: u32) {
        self.remaining_pieces.remove(&piece_index);
        self.pieces_downloading.remove(&piece_index);
    }

    fn piece_failed_download(&mut self, piece_index: u32) {
        self.pieces_downloading.remove(&piece_index);
        self.remaining_pieces.insert(piece_index);
    }

    fn last_piece_downloaded(&self) -> bool {
        self.remaining_pieces.is_empty() && self.pieces_downloading.is_empty()
    }

    pub fn listen(&mut self) -> Result<(), RecvError> {
        loop {
            let message = self.reciever.recv()?;
            match message {
                PieceManagerMessage::Stop => break,
                PieceManagerMessage::Init(_) => {
                    continue;
                }
                PieceManagerMessage::PeerPieces(peer_id, bitfield) => {
                    self.bitfields.insert(peer_id, bitfield);
                }
                PieceManagerMessage::SuccessfulDownload(piece_index) => {
                    self.piece_succesfully_downloaded(piece_index);
                    self.ui_message_sender.send_downloaded_piece();
                }
                PieceManagerMessage::FailedDownload(piece_index) => {
                    self.piece_failed_download(piece_index);
                }
            }
            if self.last_piece_downloaded() {
                // peer_connection_manager.terminate_connections_and_piece_saver();
                break;
            }
        }
        Ok(())
    }
}
