use super::errors::OpenPeerConnectionError;
use super::sender::*;
use super::worker::*;
use crate::metainfo::Metainfo;
use crate::peer::*;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use std::sync::mpsc;

pub enum OpenPeerConnectionMessage {
    //Tells worker to request a piece to peer, and contains said piece's index
    DownloadPiece(u32),
    //Orders worker to send bitfield via piece manager sender
    SendBitfield,
    //Orders worker to close connection with peer
    CloseConnection,
}

#[allow(dead_code)]
//Creates Sender and Worker for OpenPeerConnection. Opens connection with received peer
//before returning.
pub fn new_open_peer_connection(
    peer: &Peer,
    piece_manager_sender: PieceManagerSender,
    piece_saver_sender: PieceSaverSender,
    metainfo: &Metainfo,
    client_peer_id: &[u8],
) -> Result<(OpenPeerConnectionSender, OpenPeerConnectionWorker), OpenPeerConnectionError> {
    let peer_message_stream = PeerMessageService::connect_to_peer(peer)?;
    let mut connection = PeerConnection::new(
        peer,
        client_peer_id,
        metainfo,
        Box::new(peer_message_stream),
    );
    connection.open_connection()?;
    let (tx, rx) = mpsc::channel();
    Ok((
        OpenPeerConnectionSender { sender: tx },
        OpenPeerConnectionWorker {
            receiver: rx,
            connection,
            piece_manager_sender,
            piece_saver_sender,
        },
    ))
}
