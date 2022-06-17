use super::OpenPeerConnectionError;
use crate::metainfo::Metainfo;
use crate::peer::*;
use crate::piece_manager::PieceManager;
use crate::piece_saver::PieceSaver;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread::JoinHandle;

pub enum OpenPeerConnectionMessage {
    ///Comes with piece index
    DownloadPiece(u32),
    ///Informs that peer bitfield should be sent to piece manager
    SendBitfield,
    ///Informs connnection with peer should be closed
    CloseConnection,
}

pub struct OpenPeerConnection {
    sender: Sender<OpenPeerConnectionMessage>,
}

#[allow(dead_code)]
impl OpenPeerConnection {
    pub fn new(
        peer: &Peer,
        piece_manager: PieceManager,
        piece_saver: PieceSaver,
        metainfo: &Metainfo,
        client_peer_id: &[u8],
    ) -> Result<(Self, JoinHandle<()>), OpenPeerConnectionError> {
        let peer_message_stream = PeerMessageService::connect_to_peer(peer)?;
        let mut peer_connection = PeerConnection::new(
            peer,
            client_peer_id,
            metainfo,
            Box::new(peer_message_stream),
        );
        peer_connection.open_connection()?;
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            let _ = Self::listen(rx, peer_connection, piece_manager, piece_saver);
        });

        Ok((Self { sender: tx }, handle))
    }

    fn close_connection(&self) {
        let _ = self.sender.send(OpenPeerConnectionMessage::CloseConnection);
    }

    fn inform_pieces(&self) {
        let _ = self.sender.send(OpenPeerConnectionMessage::SendBitfield);
    }

    fn inform_pieces_worker_side(_connection: &PeerConnection, _piece_manager: &PieceManager) {
        //Method yet to be implemented by piece manager
        //_piece_manager.peer_pieces(_connection.get_peer_id(), _connection.get_bitfield());
    }

    fn download_piece(&self, piece_index: u32) {
        let _ = self
            .sender
            .send(OpenPeerConnectionMessage::DownloadPiece(piece_index));
    }

    fn download_piece_worker_side(
        connection: &mut PeerConnection,
        piece_index: u32,
        _piece_saver: &PieceSaver,
    ) {
        const BLOCK_SIZE: u32 = 16 * u32::pow(2, 10);
        let _piece_data: Vec<u8> = connection
            .request_piece(piece_index, BLOCK_SIZE)
            .map_err(|_| {
                PeerConnectionError::PieceRequestingError(
                    "Error trying to request piece".to_string(),
                )
            })
            .unwrap();

        //Method yet to be implemented by piece saver
        //piece_saver.validate_and_save_piece(piece_index, piece_data);
    }

    fn listen(
        receiver: Receiver<OpenPeerConnectionMessage>,
        mut connection: PeerConnection,
        piece_manager: PieceManager,
        piece_saver: PieceSaver,
    ) -> Result<(), RecvError> {
        loop {
            let message = receiver.recv()?;
            match message {
                OpenPeerConnectionMessage::SendBitfield => {
                    OpenPeerConnection::inform_pieces_worker_side(&connection, &piece_manager)
                }
                OpenPeerConnectionMessage::DownloadPiece(piece_index) => {
                    OpenPeerConnection::download_piece_worker_side(
                        &mut connection,
                        piece_index,
                        &piece_saver,
                    )
                }
                OpenPeerConnectionMessage::CloseConnection => break,
            }
        }
        Ok(())
    }
}
