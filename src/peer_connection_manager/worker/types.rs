use crate::logger::CustomLogger;
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
const LOGGER: CustomLogger = CustomLogger::init("Peer Connection Manager");
// use arc and mutex
use std::sync::Arc;
use std::sync::Mutex;
pub const FIRST_MIN_CONNECTIONS: usize = 3;

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
    fn open_connection_from_peer(
        peer: Peer,
        piece_manager_sender: PieceManagerSender,
        piece_saver_sender: PieceSaverSender,
        metainfo: Metainfo,
        client_peer_id: &[u8],
        ui_message_sender: UIMessageSender,
    ) -> Result<(OpenPeerConnectionSender, JoinHandle<()>), OpenPeerConnectionError> {
        let (open_peer_connection_sender, mut open_peer_connection_worker) =
            new_open_peer_connection(
                peer,
                piece_manager_sender,
                piece_saver_sender,
                &metainfo,
                client_peer_id,
                ui_message_sender,
            )?;

        let handle = std::thread::spawn(move || {
            open_peer_connection_worker.listen().unwrap();
        });

        open_peer_connection_sender.send_bitfield();
        Ok((open_peer_connection_sender, handle))
    }

    pub fn start_peer_connections(&mut self, peers: Vec<Peer>) {
        LOGGER.info(format!(
            "Attempting connections with {:?} peers",
            peers.len()
        ));
        let mut connection_attempts = vec![];
        let open_peer_connections = Arc::new(Mutex::new(HashMap::new()));

        for peer in peers {
            let piece_manager_sender = self.piece_manager_sender.clone();
            let piece_saver_sender = self.piece_saver_sender.clone();
            let metainfo = self.metainfo.clone();
            let client_peer_id = self.client_peer_id.clone();
            let ui_message_sender = self.ui_message_sender.clone();
            let open_peer_connections = open_peer_connections.clone();
            connection_attempts.push(std::thread::spawn(move || {
                LOGGER.info(format!("Attempting connection with peer {}", peer.ip));
                if let Ok((open_peer_connection_sender, handle)) = Self::open_connection_from_peer(
                    peer.clone(),
                    piece_manager_sender.clone(),
                    piece_saver_sender,
                    metainfo,
                    &client_peer_id,
                    ui_message_sender,
                ) {
                    LOGGER.info(format!("Successfully connected to peer at {:?}", peer.ip));
                    if let Ok(mut lock) = open_peer_connections.lock() {
                        info!("Adding peer connection {} to map", peer.ip);
                        lock.insert(peer.peer_id.clone(), (open_peer_connection_sender, handle));
                        if lock.len() == FIRST_MIN_CONNECTIONS {
                            piece_manager_sender.first_connections_started();
                        }
                    }
                }
            }));
        }
        for connection_attempt in connection_attempts {
            let _ = connection_attempt.join();
        }

        let lock = Arc::try_unwrap(open_peer_connections)
            .expect("no one should have a reference to open_peer_connections");
        self.open_peer_connections = lock
            .into_inner()
            .expect("should be able to lock open_peer_connections");
        LOGGER.info(format!(
            "Connected successfully to {:?} peers",
            self.open_peer_connections.len()
        ));
        self.piece_manager_sender.finished_stablishing_connections();
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
        self.piece_saver_sender.stop_saving();
    }

    pub fn listen(self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            trace!("Peer connection manager received message: {:?}", message);

            match message {
                PeerConnectionManagerMessage::CloseConnections => {
                    trace!("Closing connections");
                    self.close_connections();
                    break;
                }
                PeerConnectionManagerMessage::DownloadPiece(peer_id, piece_index) => {
                    self.download_piece(peer_id, piece_index)
                }
            }
            info!(
                "Total of connected peers: {}",
                self.open_peer_connections.len()
            );
        }
        Ok(())
    }
}
