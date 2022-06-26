use super::sender::types::PieceManagerSender;
use super::worker::types::PieceManagerWorker;
use crate::peer::Bitfield;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::ui::UIMessageSender;

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc;
pub enum PieceManagerMessage {
    PeerPieces(Vec<u8>, Bitfield),
    Init(PeerConnectionManagerSender),
    SuccessfulDownload(u32),
    FailedDownload(u32),
    Stop,
}

pub fn new_piece_manager(
    number_of_pieces: u32,
    ui_message_sender: UIMessageSender,
) -> (PieceManagerSender, PieceManagerWorker) {
    let (tx, rx) = mpsc::channel();

    // Initialize the peers_per_piece HashMap with empty vectors
    let mut peers_per_piece: HashMap<u32, Vec<Vec<u8>>> = HashMap::new();
    for i in 0..number_of_pieces {
        let vec: Vec<Vec<u8>> = Vec::new();
        peers_per_piece.insert(i, vec);
    }

    // Initialize remaining_pieces HashSet with all pieces
    let mut remaining_pieces: HashSet<u32> = HashSet::new();
    for i in 0..number_of_pieces {
        remaining_pieces.insert(i);
    }

    (
        PieceManagerSender { sender: tx },
        PieceManagerWorker {
            reciever: rx,
            bitfields: HashMap::new(),
            remaining_pieces,
            pieces_downloading: HashSet::new(),
            peers_per_piece,
            ui_message_sender,
        },
    )
}
