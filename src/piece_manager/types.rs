use super::sender::types::PieceManagerSender;
use super::worker::types::PieceManagerWorker;
use crate::peer::Bitfield;
use crate::peer_connection_manager::PeerConnectionManager;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc;
pub enum PieceManagerMessage {
    PeerPieces(Vec<u8>, Bitfield),
    Init(PeerConnectionManager),
    SuccessfulDownload(u32),
    FailedDownload(u32),
    Stop,
}

pub fn new_piece_manager() -> (PieceManagerSender, PieceManagerWorker) {
    let (tx, rx) = mpsc::channel();
    (
        PieceManagerSender { sender: tx },
        PieceManagerWorker {
            reciever: rx,
            bitfields: HashMap::new(),
            remaining_pieces: HashSet::new(),
            pieces_downloading: HashSet::new(),
        },
    )
}
