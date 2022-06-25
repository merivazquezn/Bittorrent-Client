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
    ui_message_sender: UIMessageSender,
) -> (PieceManagerSender, PieceManagerWorker) {
    let (tx, rx) = mpsc::channel();
    (
        PieceManagerSender { sender: tx },
        PieceManagerWorker {
            reciever: rx,
            bitfields: HashMap::new(),
            remaining_pieces: HashSet::new(),
            pieces_downloading: HashSet::new(),
            ui_message_sender,
        },
    )
}
