use super::errors::OpenPeerConnectionError;
use super::sender::*;
use super::worker::*;
use crate::metainfo::Metainfo;
use crate::peer::*;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use crate::ui::UIMessageSender;
use std::sync::mpsc;

#[derive(Debug, Clone)]
pub enum OpenPeerConnectionMessage {
    //Tells worker to request a piece to peer, and contains said piece's index
    DownloadPiece(u32),
    //Orders worker to send bitfield via piece manager sender
    SendBitfield,
    //Orders worker to close connection with peer
    CloseConnection,
}

//Creates Sender and Worker for OpenPeerConnection. Opens connection with received peer
//before returning.
pub fn new_open_peer_connection(
    peer: Peer,
    piece_manager_sender: PieceManagerSender,
    piece_saver_sender: PieceSaverSender,
    peer_connection_manager_sender: PeerConnectionManagerSender,
    metainfo: &Metainfo,
    client_peer_id: &[u8],
    ui_message_sender: UIMessageSender,
) -> Result<(OpenPeerConnectionSender, OpenPeerConnectionWorker), OpenPeerConnectionError> {
    let peer_message_stream = peer.connect()?;
    let mut connection = PeerConnection::new(
        peer,
        client_peer_id,
        metainfo,
        peer_message_stream,
        ui_message_sender,
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
            peer_connection_manager_sender,
            failed_download_in_a_row: 0,
            is_open: true,
        },
    ))
}
