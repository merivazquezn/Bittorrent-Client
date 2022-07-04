use crate::logger::CustomLogger;
use crate::metainfo::Metainfo;
use crate::peer::*;
use crate::peer_connection_manager::types::PeerConnectionManagerMessage;
use crate::peer_connection_manager::{open_peer_connection::*, PeerConnectionManagerSender};
use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use crate::tracker::ITrackerService;
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
    sender: OpenPeerConnectionSender,
    handle: JoinHandle<()>,
    is_open: bool,
}

pub struct PeerConnectionManagerWorker {
    pub receiver: Receiver<PeerConnectionManagerMessage>,
    pub piece_manager_sender: PieceManagerSender,
    pub piece_saver_sender: PieceSaverSender,
    pub peer_connections: HashMap<Vec<u8>, PeerConnection>,
    pub metainfo: Metainfo,
    pub client_peer_id: Vec<u8>,
    pub ui_message_sender: UIMessageSender,
    pub tracker_request_count: u32,
    pub last_time_requested: Instant,
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
            open_peer_connection_worker.listen().unwrap();
            // set connection closed

            // ui_message_sender.send_closed_connection();
        });

        open_peer_connection_sender.send_bitfield();
        Ok((open_peer_connection_sender, handle))
    }

    fn open_peer_connection_count(&self) -> usize {
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

    fn update_peer_connections(
        &mut self,
        tracker_service: &mut Box<dyn ITrackerService>,
        peer_connection_manager_sender: PeerConnectionManagerSender,
    ) -> usize {
        let mut new_connections = 0;
        if let Ok(tracker_response) = tracker_service.get_response() {
            for peer in tracker_response.peers {
                if !self.peer_connections.contains_key(&peer.peer_id) {
                    match Self::open_connection_from_peer(
                        peer.clone(),
                        self.piece_manager_sender.clone(),
                        self.piece_saver_sender.clone(),
                        peer_connection_manager_sender.clone(),
                        self.metainfo.clone(),
                        &self.client_peer_id,
                        self.ui_message_sender.clone(),
                    ) {
                        Ok((open_peer_connection_sender, handle)) => {
                            self.peer_connections.insert(
                                peer.peer_id.clone(),
                                PeerConnection {
                                    sender: open_peer_connection_sender,
                                    handle,
                                    is_open: true,
                                },
                            );
                            self.ui_message_sender.send_new_connection();
                            new_connections += 1;
                        }
                        Err(error) => {
                            error!("Error opening peer connection: {:?}", error);
                        }
                    }
                }
            }
        } else {
            error!("Tracker rejected client's request");
        }
        new_connections
    }

    pub fn start_peer_connections(
        &mut self,
        peers: Vec<Peer>,
        peer_connection_manager_sender: PeerConnectionManagerSender,
    ) {
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
            let peer_connection_manager_sender_clone = peer_connection_manager_sender.clone();
            connection_attempts.push(std::thread::spawn(move || {
                LOGGER.info(format!("Attempting connection with peer {}", peer.ip));
                if let Ok((open_peer_connection_sender, handle)) = Self::open_connection_from_peer(
                    peer.clone(),
                    piece_manager_sender.clone(),
                    piece_saver_sender,
                    peer_connection_manager_sender_clone,
                    metainfo,
                    &client_peer_id,
                    ui_message_sender,
                ) {
                    LOGGER.info(format!("Successfully connected to peer at {:?}", peer.ip));
                    if let Ok(mut lock) = open_peer_connections.lock() {
                        info!("Adding peer connection {} to map", peer.ip);
                        lock.insert(
                            peer.peer_id.clone(),
                            PeerConnection {
                                sender: open_peer_connection_sender,
                                handle,
                                is_open: true,
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
            Some(interval) => {
                let now = Instant::now();
                warn!(
                    "{:?} > {:?}",
                    now.duration_since(self.last_time_requested),
                    interval
                );
                if now.duration_since(self.last_time_requested) > interval {
                    self.last_time_requested = now;
                    true
                } else {
                    false
                }
            }
            None => true,
        }
    }

    fn able_to_reach_tracker_again(&mut self, interval: Option<Duration>) -> bool {
        warn!(
            "{:?} < {:?}, {:?} < {:?}",
            self.tracker_request_count,
            MAX_TRACKER_REQUESTS,
            self.open_peer_connection_count(),
            MIN_CONNECTIONS
        );
        self.tracker_request_count < MAX_TRACKER_REQUESTS
            && self.open_peer_connection_count() < MIN_CONNECTIONS
            && self.interval_long_enough(interval)
    }

    pub fn listen(
        mut self,
        mut tracker_service: Box<dyn ITrackerService>,
        interval: Option<Duration>,
        peer_connection_manager_sender: PeerConnectionManagerSender,
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
                        self.download_piece(peer_id, piece_index);
                    } else {
                        self.piece_manager_sender.failed_connection(peer_id);
                    }
                }

                PeerConnectionManagerMessage::FailedConnection(peer_id) => {
                    self.set_peer_connection_to_closed(peer_id.clone());
                    if self.able_to_reach_tracker_again(interval) {
                        info!("sending reaked tracker request");
                        self.piece_manager_sender.reasked_tracker();
                        self.piece_manager_sender.failed_connection(peer_id.clone());
                        info!("sent reaked tracker request");
                        let new_connections = self.update_peer_connections(
                            &mut tracker_service,
                            peer_connection_manager_sender.clone(),
                        );
                        warn!("New connections: {:?}", new_connections);
                        self.piece_manager_sender
                            .finished_stablishing_connections(new_connections);
                        self.tracker_request_count += 1;
                    } else {
                        self.piece_manager_sender.failed_connection(peer_id.clone());
                    }
                    self.ui_message_sender.send_closed_connection(peer_id);
                }
            }
            info!("Total of connected peers: {}", self.peer_connections.len());
        }
        Ok(())
    }
}
