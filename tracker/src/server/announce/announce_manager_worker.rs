use super::constants::{INTERVAL_IN_SECONDS, MAX_RESPONSE_PEERS, TRACKER_ID};
use super::types::ActivePeers;
use super::types::Peer;
use super::types::PeerEntry;
use super::types::TrackerResponse;
use super::utils::is_active_peer;
use super::utils::is_peer_stopping;
use super::utils::peer_is_seeder;
use super::{AnnounceMessage, TrackerEvent};
use crate::aggregator::AggregatorSender;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

pub struct AnnounceManagerWorker {
    peers_by_torrent: HashMap<Vec<u8>, ActivePeers>,
    receiver: Receiver<AnnounceMessage>,
    aggregator: AggregatorSender,
}

impl AnnounceManagerWorker {
    pub fn new(receiver: Receiver<AnnounceMessage>, aggregator_sender: AggregatorSender) -> Self {
        AnnounceManagerWorker {
            peers_by_torrent: HashMap::new(),
            receiver,
            aggregator: aggregator_sender,
        }
    }

    pub fn listen(mut self) -> Result<(), RecvError> {
        loop {
            let message: AnnounceMessage = self.receiver.recv()?;
            match message {
                AnnounceMessage::Announce(announce_request, sender) => {
                    let info_hash: Vec<u8> = announce_request.info_hash.clone();
                    let peer: Peer = Peer {
                        ip: announce_request.ip.clone(),
                        port: announce_request.port,
                        peer_id: announce_request.peer_id.clone(),
                    };
                    let is_seeder: bool = peer_is_seeder(&announce_request);
                    let is_stopping: bool = is_peer_stopping(&announce_request);
                    if announce_request.event == TrackerEvent::Completed {
                        let info_hash_str = match String::from_utf8(info_hash.clone()) {
                            Ok(s) => s,
                            Err(_) => {
                                println!("Invalid info hash");
                                continue;
                            }
                        };
                        self.aggregator
                            .increment(format!("{}.complete_download_peers", info_hash_str));
                    }

                    let announce_res = self.handle_announce(
                        info_hash,
                        peer,
                        announce_request.ip.clone(),
                        announce_request.port,
                        is_seeder,
                        is_stopping,
                    );
                    self = announce_res.0;
                    let response: TrackerResponse = announce_res.1;
                    sender.send(response).unwrap();
                }
                AnnounceMessage::Stop => break,
            }
        }

        Ok(())
    }

    fn handle_announce(
        self,
        info_hash: Vec<u8>,
        peer: Peer,
        ip: String,
        port: u16,
        is_seeder: bool,
        is_stopping: bool,
    ) -> (Self, TrackerResponse) {
        if self.torrent_already_exists(&info_hash) {
            self.get_peers(info_hash, ip, port, peer.peer_id, is_seeder, is_stopping)
        } else {
            self.add_new_torrent(info_hash, ip, port, peer.peer_id, is_seeder)
        }
    }

    fn get_peers(
        mut self,
        info_hash: Vec<u8>,
        ip: String,
        port: u16,
        peer_id: Vec<u8>,
        is_seeder: bool,
        is_stopping: bool,
    ) -> (Self, TrackerResponse) {
        let sender_peer: Peer = Peer { ip, port, peer_id };

        let mut seeder_count: u32 = 0;
        let mut leecher_count: u32 = 0;
        let mut active_peers: Vec<Peer> = Vec::new();
        let mut is_existing_peer = false;

        for (i, torrent_peer_entry) in self
            .peers_by_torrent
            .get_mut(&info_hash)
            .unwrap()
            .peers
            .iter_mut()
            .enumerate()
        {
            if sender_peer.ip == torrent_peer_entry.peer.ip {
                torrent_peer_entry.last_announce = Local::now();
                torrent_peer_entry.is_seeder = is_seeder;
                is_existing_peer = true;
                if is_stopping {
                    active_peers.remove(i);
                }

                continue;
            }

            if is_active_peer(torrent_peer_entry.last_announce) {
                if torrent_peer_entry.is_seeder {
                    seeder_count += 1;
                } else {
                    leecher_count += 1;
                }

                if active_peers.len() >= MAX_RESPONSE_PEERS {
                    continue;
                }
                active_peers.push(torrent_peer_entry.peer.clone());
            } else {
                active_peers.remove(i);
            }
        }

        if !is_existing_peer || !is_stopping {
            self.peers_by_torrent
                .get_mut(&info_hash)
                .unwrap()
                .peers
                .push(PeerEntry {
                    peer: sender_peer,
                    last_announce: Local::now(),
                    is_seeder,
                })
        }

        let key: String = format!("{}.active_peers", String::from_utf8(info_hash).unwrap());
        self.aggregator.set(key, active_peers.len() as i32);

        let response: TrackerResponse = TrackerResponse {
            interval_in_seconds: INTERVAL_IN_SECONDS,
            tracker_id: String::from(TRACKER_ID),
            complete: seeder_count,
            incomplete: leecher_count,
            peers: active_peers,
        };

        (self, response)
    }

    fn add_new_torrent(
        mut self,
        info_hash: Vec<u8>,
        ip: String,
        port: u16,
        peer_id: Vec<u8>,
        is_seeder: bool,
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

        let key: String = format!("{}.active_peers", String::from_utf8(info_hash).unwrap());
        self.aggregator.set(key, 1);

        let response: TrackerResponse = TrackerResponse {
            interval_in_seconds: INTERVAL_IN_SECONDS,
            tracker_id: String::from(TRACKER_ID),
            complete: 0,
            incomplete: 0,
            peers: Vec::new(),
        };

        (self, response)
    }

    fn torrent_already_exists(&self, info_hash: &Vec<u8>) -> bool {
        self.peers_by_torrent.contains_key(info_hash)
    }
}
