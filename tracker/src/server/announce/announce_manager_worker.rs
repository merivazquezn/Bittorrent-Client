use super::constants::TRACKER_ID;
use super::types::ActivePeers;
use super::types::Peer;
use super::types::PeerEntry;
use super::types::TrackerResponse;
use super::utils::has_completed;
use super::utils::is_active_peer;
use super::utils::is_peer_stopping;
use super::AnnounceMessage;
use crate::aggregator::AggregatorSender;
use crate::application_constants::{ACTIVE_PEERS_STAT, COMPLETED_DOWNLOADS_STAT, TORRENTS_STAT};
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

/// The announce manager worker is responsible of diciding which peers
/// are active, and stores the active peers for each torrent.
/// When a new peer announces, the announce manager worker will
/// return the list of active peers and other response parameters.
pub struct AnnounceManagerWorker {
    /// The key is the torrent's info_hash, the value is currently active peers of that torrent
    peers_by_torrent: HashMap<Vec<u8>, ActivePeers>,
    /// For receiving AnnounceMessages from announcing peers
    receiver: Receiver<AnnounceMessage>,
    /// So that the AnnounceManager can send events metrics to the aggregator
    aggregator: AggregatorSender,
    /// Interval in seconds tha peers hace to wait between requests
    interval: u32,
}

impl AnnounceManagerWorker {
    /// Creates a new AnnounceManagerWorker
    pub fn new(
        receiver: Receiver<AnnounceMessage>,
        aggregator_sender: AggregatorSender,
        interval: u32,
    ) -> Self {
        AnnounceManagerWorker {
            peers_by_torrent: HashMap::new(),
            receiver,
            aggregator: aggregator_sender,
            interval,
        }
    }

    /// Starts the announce manager worker
    ///
    /// # Returns:
    /// ## On Success:
    /// - Nothing
    ///
    /// ## On Failure:
    /// - A `RecvError` if the receiver fails to receive messages
    ///
    /// The announce manager starts listening for messages, which can:
    /// - Stop the AnnounceManager
    ///
    /// - Announce a new peer
    ///
    /// - Remove inactive peers
    ///
    /// In the announce case, the Announce Manager will also
    /// remove all of that torrent inactive peers.
    /// It will also trigger the active_peers, completed_downloads
    /// and amonunt of torrents events when necessry.
    pub fn listen(mut self) -> Result<(), RecvError> {
        loop {
            let message: AnnounceMessage = self.receiver.recv()?;
            match message {
                AnnounceMessage::Announce(announce_request, sender, interval) => {
                    let info_hash: Vec<u8> = announce_request.info_hash.clone();
                    let peer: Peer = Peer {
                        ip: announce_request.ip.clone(),
                        port: announce_request.port,
                        peer_id: announce_request.peer_id.clone(),
                    };
                    let has_completed: bool = has_completed(&announce_request);
                    let is_stopping: bool = is_peer_stopping(&announce_request);

                    let announce_res = self.handle_announce(
                        info_hash,
                        peer,
                        (announce_request.ip.clone(), announce_request.port),
                        has_completed,
                        is_stopping,
                        interval,
                    );
                    self = announce_res.0;
                    let response: TrackerResponse = announce_res.1;
                    sender.send(response).unwrap();
                }
                AnnounceMessage::Update => self.remove_all_inactive_peers(),
                AnnounceMessage::Stop => break,
            }
        }

        Ok(())
    }

    fn remove_all_inactive_peers(&mut self) {
        println!("removing all inactive peers due to timer update");
        let peer_hashmap_clone = self.peers_by_torrent.clone();
        for (info_hash, _) in peer_hashmap_clone {
            self.remove_inactive_peers(&info_hash, self.interval);
            let key: String = format!(
                "{}.active_peers",
                String::from_utf8(info_hash.clone()).unwrap()
            );
            self.aggregator.set(
                key,
                self.peers_by_torrent
                    .get_mut(&info_hash.clone())
                    .unwrap()
                    .peers
                    .len()
                    .try_into()
                    .unwrap(),
            );
        }
    }

    fn handle_announce(
        mut self,
        info_hash: Vec<u8>,
        peer: Peer,
        ipport: (String, u16),
        has_completed: bool,
        is_stopping: bool,
        interval: u32,
    ) -> (Self, TrackerResponse) {
        if self.torrent_already_exists(&info_hash) {
            self.remove_inactive_peers(&info_hash, interval);
            if is_stopping {
                self.remove_peer(&info_hash, peer.peer_id.clone());
            } else {
                self.add_peer_if_not_in_list(&info_hash, peer.clone(), has_completed);
                self.update_peer_if_in_list(&info_hash, peer.clone(), has_completed);
            }
            let key: String = format!(
                "{}.active_peers",
                String::from_utf8(info_hash.clone()).unwrap()
            );
            self.aggregator.set(
                key,
                self.peers_by_torrent
                    .get_mut(&info_hash.clone())
                    .unwrap()
                    .peers
                    .len()
                    .try_into()
                    .unwrap(),
            );

            let response = self.build_tracker_response(info_hash, &peer.peer_id, interval);
            (self, response)
        } else {
            self.add_new_torrent(
                info_hash,
                ipport.0,
                ipport.1,
                peer.peer_id,
                has_completed,
                interval,
            )
        }
    }

    fn remove_inactive_peers(&mut self, info_hash: &[u8], interval: u32) {
        let active_peers = &mut self.peers_by_torrent.get_mut(info_hash).unwrap().peers;
        self.peers_by_torrent.get_mut(info_hash).unwrap().peers = active_peers
            .iter()
            .filter(|peer| is_active_peer(peer.last_announce, interval))
            .cloned()
            .collect();
    }

    fn remove_peer(&mut self, info_hash: &[u8], sender_peer_id: Vec<u8>) {
        let active_peers = &mut self.peers_by_torrent.get_mut(info_hash).unwrap().peers;
        self.peers_by_torrent.get_mut(info_hash).unwrap().peers = active_peers
            .iter()
            .filter(|peer_entry| peer_entry.peer.peer_id != sender_peer_id)
            .cloned()
            .collect();
    }

    fn add_peer_if_not_in_list(&mut self, info_hash: &[u8], peer: Peer, _is_seeder: bool) {
        let active_peers = &mut self.peers_by_torrent.get_mut(info_hash).unwrap().peers;
        if !active_peers
            .iter()
            .any(|peer_entry| peer_entry.peer.peer_id == peer.peer_id)
        {
            active_peers.push(PeerEntry {
                is_seeder: false,
                peer,
                last_announce: Local::now(),
            });
        }
    }

    fn update_peer_if_in_list(&mut self, info_hash: &[u8], peer: Peer, has_completed: bool) {
        let active_peers = &mut self.peers_by_torrent.get_mut(info_hash).unwrap().peers;
        for peer_entry in active_peers.iter_mut() {
            if peer_entry.peer.peer_id == peer.peer_id {
                if has_completed && !peer_entry.is_seeder {
                    self.aggregator.increment(format!(
                        "{}.complete_download_peers",
                        String::from_utf8(info_hash.to_vec()).unwrap()
                    ));
                }
                peer_entry.is_seeder = has_completed;
                peer_entry.last_announce = Local::now();
                break;
            }
        }
    }

    fn get_active_peers_iter(&self, info_hash: &[u8]) -> std::slice::Iter<'_, PeerEntry> {
        // This unwrap shouldn't fail, because we already checked that the torrent exists
        self.peers_by_torrent.get(info_hash).unwrap().peers.iter()
    }

    /// Builds the tracker response struct, so that the main connection thread will
    /// send it to the peer.
    fn build_tracker_response(
        &self,
        info_hash: Vec<u8>,
        sender_peer_id: &[u8],
        interval: u32,
    ) -> TrackerResponse {
        let seeder_count = self
            .get_active_peers_iter(&info_hash)
            .filter(|peer_entry| peer_entry.is_seeder && peer_entry.peer.peer_id != sender_peer_id)
            .count();
        let active_peers_excluding_sender: Vec<Peer> = self
            .get_active_peers_iter(&info_hash)
            .filter(|peer_entry| peer_entry.peer.peer_id != sender_peer_id)
            .map(|peer_entry| peer_entry.peer.clone())
            .collect();

        TrackerResponse {
            interval_in_seconds: interval,
            tracker_id: String::from(TRACKER_ID),
            complete: seeder_count as u32,
            incomplete: (active_peers_excluding_sender.len() - seeder_count) as u32,
            peers: active_peers_excluding_sender,
        }
    }

    fn add_new_torrent(
        mut self,
        info_hash: Vec<u8>,
        ip: String,
        port: u16,
        peer_id: Vec<u8>,
        is_seeder: bool,
        interval: u32,
    ) -> (Self, TrackerResponse) {
        let peer: Peer = Peer { ip, port, peer_id };

        let new_active_peers: ActivePeers = ActivePeers {
            peers: vec![PeerEntry {
                peer,
                last_announce: Local::now(),
                is_seeder,
            }],
        };

        self.peers_by_torrent
            .insert(info_hash.clone(), new_active_peers);

        let active_peer_stats_key: String = format!(
            "{}.{}",
            String::from_utf8(info_hash.clone()).unwrap(),
            ACTIVE_PEERS_STAT
        );
        self.aggregator.set(active_peer_stats_key, 1);

        let complete_downloads_key: String = format!(
            "{}.{}",
            String::from_utf8(info_hash).unwrap(),
            COMPLETED_DOWNLOADS_STAT
        );
        self.aggregator.set(complete_downloads_key, 1);

        self.aggregator.increment(TORRENTS_STAT.to_string());

        let response: TrackerResponse = TrackerResponse {
            interval_in_seconds: interval,
            tracker_id: String::from(TRACKER_ID),
            complete: 0,
            incomplete: 0,
            peers: Vec::new(),
        };

        (self, response)
    }

    fn torrent_already_exists(&self, info_hash: &[u8]) -> bool {
        self.peers_by_torrent.contains_key(info_hash)
    }
}
