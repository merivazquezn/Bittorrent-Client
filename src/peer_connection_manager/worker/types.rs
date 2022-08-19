use crate::logger::CustomLogger;
use crate::metainfo::Metainfo;
use crate::peer::*;
use crate::peer_connection_manager::types::PeerConnectionManagerMessage;
use crate::peer_connection_manager::{open_peer_connection::*, PeerConnectionManagerSender};
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use crate::tracker::ITrackerServiceV2;
use crate::ui::UIMessageSender;
use log::*;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvError};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

const LOGGER: CustomLogger = CustomLogger::init("Peer Connection Manager");

pub const FIRST_MIN_CONNECTIONS: usize = 2;
pub const MAX_TRACKER_REQUESTS: u32 = 3;
pub const MIN_CONNECTIONS: usize = 10;

#[derive(Debug)]
pub struct PeerConnection {
    peer: Peer,
    sender: OpenPeerConnectionSender,
    handle: JoinHandle<()>,
    is_open: bool,
    piece_request_count: u32,
}

pub struct PeerConnectionManagerWorker {
    pub receiver: Receiver<PeerConnectionManagerMessage>,
    pub piece_manager_sender: PieceManagerSender,
    pub piece_saver_sender: PieceSaverSender,
    pub peer_connections: HashMap<Vec<u8>, PeerConnection>,
    pub metainfo: Metainfo,
    pub client_peer_id: Vec<u8>,
    pub ui_message_sender: UIMessageSender,
    pub last_announce: Instant,
}

impl PeerConnectionManagerWorker {
    fn open_connection_from_peer(
        peer: Peer,
        piece_manager_sender: PieceManagerSender,
        piece_saver_sender: PieceSaverSender,
        peer_connection_manager_sender: PeerConnectionManagerSender,
        metainfo: Metainfo,
        client_peer_id: &[u8],
        ui_message_sender: UIMessageSender,
    ) -> Result<(OpenPeerConnectionSender, JoinHandle<()>), OpenPeerConnectionError> {
        let (open_peer_connection_sender, mut open_peer_connection_worker) =
            new_open_peer_connection(
                peer,
                piece_manager_sender,
                piece_saver_sender,
                peer_connection_manager_sender,
                &metainfo,
                client_peer_id,
                ui_message_sender,
            )?;

        let handle = std::thread::spawn(move || {
            if let Err((err, _)) = open_peer_connection_worker.listen() {
                LOGGER.error(err);
            }
        });

        open_peer_connection_sender.send_bitfield();
        Ok((open_peer_connection_sender, handle))
    }

    fn _open_peer_connection_count(&self) -> usize {
        self.peer_connections
            .values()
            .filter(|peer_connection| peer_connection.is_open)
            .count()
    }

    fn set_peer_connection_to_closed(&mut self, peer_id: Vec<u8>) {
        if let Some(peer_connection) = self.peer_connections.get_mut(&peer_id) {
            peer_connection.is_open = false;
        }
    }

    pub fn start_peer_connections(
        &mut self,
        peers: Vec<Peer>,
        peer_connection_manager_sender: PeerConnectionManagerSender,
    ) {
        LOGGER.info(format!(
            "Attempting connections with {:?} peers...",
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
            let peer_connection_manager_sender_clone = peer_connection_manager_sender.clone();
            connection_attempts.push(std::thread::spawn(move || {
                if let Ok((open_peer_connection_sender, handle)) = Self::open_connection_from_peer(
                    peer.clone(),
                    piece_manager_sender.clone(),
                    piece_saver_sender,
                    peer_connection_manager_sender_clone,
                    metainfo,
                    &client_peer_id,
                    ui_message_sender,
                ) {
                    if let Ok(mut lock) = open_peer_connections.lock() {
                        lock.insert(
                            peer.peer_id.clone(),
                            PeerConnection {
                                sender: open_peer_connection_sender,
                                handle,
                                is_open: true,
                                peer: peer.clone(),
                                piece_request_count: 0,
                            },
                        );
                    }
                }
            }));
        }
        for connection_attempt in connection_attempts {
            let _ = connection_attempt.join();
        }

        let lock = Arc::try_unwrap(open_peer_connections)
            .expect("no one should have a reference to open_peer_connections");
        self.peer_connections = lock
            .into_inner()
            .expect("should be able to lock open_peer_connections");
        LOGGER.info(format!(
            "Connected successfully to {:?} peers",
            self.peer_connections.len()
        ));

        self.piece_manager_sender
            .finished_stablishing_connections(self.peer_connections.len());
    }

    fn download_piece(&self, peer_id: Vec<u8>, piece_index: u32) {
        let peer_connection = self.peer_connections.get(&peer_id).unwrap();
        peer_connection.sender.download_piece(piece_index);
    }

    fn close_connections(self) {
        for (_, peer_connection) in self.peer_connections.into_iter() {
            peer_connection.sender.close_connection();
            peer_connection.handle.join().unwrap();
        }
        self.piece_saver_sender.stop_saving();
    }
    fn interval_long_enough(&mut self, interval: Option<Duration>) -> bool {
        match interval {
            Some(interval) => Instant::now().duration_since(self.last_announce) > interval,
            None => false,
        }
    }

    pub fn listen(
        mut self,
        _tracker_service: &mut impl ITrackerServiceV2,
        interval: Option<Duration>,
        _peer_connection_manager_sender: PeerConnectionManagerSender,
    ) -> Result<(), RecvError> {
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
                    if self.peer_connections[&peer_id].is_open {
                        LOGGER.debug(format!(
                            "Sending download request {} to peer {:?} with piece requests: {}",
                            piece_index,
                            self.peer_connections[&peer_id].peer.peer_id,
                            self.peer_connections[&peer_id].piece_request_count
                        ));
                        self.download_piece(peer_id.clone(), piece_index);
                        self.peer_connections
                            .get_mut(&peer_id)
                            .unwrap()
                            .piece_request_count += 1;
                    } else {
                        LOGGER.error(format!(
                            "Tried to download piece from closed peer connection {:?}, so send it back to retry",
                            peer_id
                        ));
                        self.piece_manager_sender
                            .failed_download(piece_index, peer_id);
                    }
                    // print peers that have not yet downloaded a piece
                    let mut peers = vec![];
                    for (_, peer_connection) in self.peer_connections.iter() {
                        if peer_connection.piece_request_count == 0 {
                            peers.push(peer_connection.peer.clone());
                        }
                    }

                    if self.interval_long_enough(interval) {
                        //let _ = tracker_service.announce(None);
                        self.last_announce = Instant::now();
                    }
                }

                PeerConnectionManagerMessage::FailedConnection(peer_id) => {
                    self.set_peer_connection_to_closed(peer_id.clone());
                    self.piece_manager_sender.failed_connection(peer_id);
                }
            }
        }
        Ok(())
    }
}
