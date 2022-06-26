use crate::metainfo::Metainfo;
use crate::peer::*;
use crate::peer_connection_manager::open_peer_connection::*;
use crate::peer_connection_manager::types::PeerConnectionManagerMessage;
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use crate::ui::UIMessageSender;
use log::*;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvError};
use std::thread::JoinHandle;

pub struct PeerConnectionManagerWorker {
    pub receiver: Receiver<PeerConnectionManagerMessage>,
    pub piece_manager_sender: PieceManagerSender,
    pub piece_saver_sender: PieceSaverSender,
    pub open_peer_connections: HashMap<Vec<u8>, (OpenPeerConnectionSender, JoinHandle<()>)>,
    pub metainfo: Metainfo,
    pub client_peer_id: Vec<u8>,
    pub ui_message_sender: UIMessageSender,
}

impl PeerConnectionManagerWorker {
    fn open_connection_from_peer(&self, peer: &Peer) -> (OpenPeerConnectionSender, JoinHandle<()>) {
        let (open_peer_connection_sender, mut open_peer_connection_worker) =
            new_open_peer_connection(
                peer,
                self.piece_manager_sender.clone(),
                self.piece_saver_sender.clone(),
                &self.metainfo,
                &self.client_peer_id,
                self.ui_message_sender.clone(),
            )
            .unwrap();

        let handle = std::thread::spawn(move || {
            open_peer_connection_worker.listen().unwrap();
        });

        open_peer_connection_sender.send_bitfield();
        (open_peer_connection_sender, handle)
    }
    pub fn start_peer_connections(&mut self, peers: &[Peer]) {
        info!("Starting connections with: {:?} peers", peers.len());
        for peer in peers {
            trace!("About to start connection to peer: {:?}", peer);
            let (open_peer_connection_sender, handle) = self.open_connection_from_peer(peer);
            self.open_peer_connections
                .insert(peer.peer_id.clone(), (open_peer_connection_sender, handle));
        }
    }

    fn download_piece(&self, peer_id: Vec<u8>, piece_index: u32) {
        let (peer_connection, _handle) = self.open_peer_connections.get(&peer_id).unwrap();
        peer_connection.download_piece(piece_index);
    }

    fn close_connections(self) {
        for (_id, (connection, handle)) in self.open_peer_connections.into_iter() {
            connection.close_connection();
            handle.join().unwrap();
        }
        self.piece_saver_sender.stop();
    }

    pub fn listen(self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            trace!("Peer connection manager received message: {:?}", message);
            match message {
                PeerConnectionManagerMessage::CloseConnections => {
                    self.close_connections();
                    break;
                }
                PeerConnectionManagerMessage::DownloadPiece(peer_id, piece_index) => {
                    self.download_piece(peer_id, piece_index)
                }
            }
        }
        Ok(())
    }
}
