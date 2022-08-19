use super::sender::*;
use super::worker::*;
use crate::metainfo::Metainfo;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use crate::ui::UIMessageSender;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

#[derive(Debug)]
pub enum PeerConnectionManagerMessage {
    DownloadPiece(Vec<u8>, u32),
    FailedConnection(Vec<u8>),
    CloseConnections,
}

pub fn new_peer_connection_manager(
    piece_manager_sender: PieceManagerSender,
    piece_saver_sender: PieceSaverSender,
    metainfo: &Metainfo,
    client_peer_id: &[u8],
    ui_message_sender: UIMessageSender,
) -> (PeerConnectionManagerSender, PeerConnectionManagerWorker) {
    let (tx, rx) = mpsc::channel();
    (
        PeerConnectionManagerSender { sender: tx },
        PeerConnectionManagerWorker {
            receiver: rx,
            piece_manager_sender,
            piece_saver_sender,
            peer_connections: HashMap::new(),
            metainfo: metainfo.clone(),
            client_peer_id: client_peer_id.to_vec(),
            ui_message_sender,
            last_announce: Instant::now(),
        },
    )
}
