use super::constants::{INTERVAL_IN_SECONDS, MAX_RESPONSE_PEERS, TRACKER_ID};
use super::types::ActivePeers;
use super::types::Peer;
use super::types::PeerEntry;
use super::types::TrackerResponse;
use super::utils::is_active_peer;
use super::AnnounceMessage;
use crate::http::IHttpService;
use bittorrent_rustico::bencode::encode;
use bittorrent_rustico::bencode::BencodeDecodedValue;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

pub struct AnnounceManagerWorker {
    peers_by_torrent: HashMap<Vec<u8>, ActivePeers>,
    receiver: Receiver<AnnounceMessage>,
}

impl AnnounceManagerWorker {
    pub fn new(receiver: Receiver<AnnounceMessage>) -> Self {
        AnnounceManagerWorker {
            peers_by_torrent: HashMap::new(),
            receiver,
        }
    }

    pub fn listen(mut self) -> Result<(), RecvError> {
        loop {
            let message: AnnounceMessage = self.receiver.recv()?;
            match message {
                AnnounceMessage::Announce(announce_request, http_service) => {
                    let info_hash: Vec<u8> = announce_request.info_hash.clone();
                    let peer: Peer = Peer {
                        ip: announce_request.ip.clone(),
                        port: announce_request.port,
                        peer_id: announce_request.peer_id.clone(),
                    };

                    self.handle_announce(
                        http_service,
                        info_hash,
                        peer,
                        announce_request.ip.clone(),
                    );
                }
            }
        }
    }

    fn handle_announce(
        &mut self,
        http_service: Box<dyn IHttpService>,
        info_hash: Vec<u8>,
        peer: Peer,
        ip: String,
    ) {
        if self.torrent_already_exists(&info_hash) {
            let torrent_peers: &mut ActivePeers = self.get_peers_from_torrent(&info_hash);
            Self::get_peers_and_send_response(torrent_peers, ip, peer.peer_id, http_service);
        } else {
            self.add_new_torrent_and_send_response(info_hash, ip, peer.peer_id, http_service);
        }
    }

    fn get_peers_and_send_response(
        torrent_peers: &mut ActivePeers,
        ip: String,
        peer_id: Vec<u8>,
        http_service: Box<dyn IHttpService>,
    ) {
        let sender_peer: Peer = Peer {
            ip,
            port: http_service.get_client_address().port(),
            peer_id,
        };

        let mut active_peers: Vec<Peer> = Vec::new();
        let mut is_existing_peer = false;
        for (i, torrent_peer_entry) in torrent_peers.peers.iter_mut().enumerate() {
            if sender_peer.ip == torrent_peer_entry.peer.ip {
                torrent_peer_entry.last_announce = Local::now();
                is_existing_peer = true;
                continue;
            }

            if is_active_peer(torrent_peer_entry.last_announce) {
                if active_peers.len() >= MAX_RESPONSE_PEERS {
                    continue;
                }
                active_peers.push(torrent_peer_entry.peer.clone());
            } else {
                active_peers.remove(i);
            }
        }

        if !is_existing_peer {
            torrent_peers.peers.push(PeerEntry {
                peer: sender_peer,
                last_announce: Local::now(),
            })
        }

        let response: TrackerResponse = TrackerResponse {
            interval_in_seconds: INTERVAL_IN_SECONDS,
            tracker_id: String::from(TRACKER_ID),
            complete: 0,
            incomplete: 0,
            peers: active_peers,
        };

        Self::send_response(http_service, response);
    }

    fn add_new_torrent_and_send_response(
        &mut self,
        info_hash: Vec<u8>,
        ip: String,
        peer_id: Vec<u8>,
        http_service: Box<dyn IHttpService>,
    ) {
        let peer: Peer = Peer {
            ip,
            port: http_service.get_client_address().port(),
            peer_id,
        };

        let new_active_peers: ActivePeers = ActivePeers {
            peers: vec![PeerEntry {
                peer,
                last_announce: Local::now(),
            }],
        };

        self.peers_by_torrent.insert(info_hash, new_active_peers);
        let response: TrackerResponse = TrackerResponse {
            interval_in_seconds: INTERVAL_IN_SECONDS,
            tracker_id: String::from(TRACKER_ID),
            complete: 0,
            incomplete: 0,
            peers: Vec::new(),
        };
        Self::send_response(http_service, response);
    }

    fn torrent_already_exists(&self, info_hash: &Vec<u8>) -> bool {
        self.peers_by_torrent.contains_key(info_hash)
    }

    fn get_peers_from_torrent(&mut self, info_hash: &Vec<u8>) -> &mut ActivePeers {
        self.peers_by_torrent.get_mut(info_hash).unwrap()
    }

    fn send_response(mut http_service: Box<dyn IHttpService>, response: TrackerResponse) {
        let response_bytes: Vec<u8> = Self::get_response_bytes(response);
        match http_service.send_ok_response(response_bytes, "application/octet-stream".to_string())
        {
            Ok(()) => println!("Torrent added successfully"),
            Err(err) => println!(
                "Error sending ok response while adding new torrent: {:?}",
                err
            ),
        };
    }

    /// Encodes with bencoding the tracker response, and returns the bytes to be sent
    fn get_response_bytes(response: TrackerResponse) -> Vec<u8> {
        let mut response_map: HashMap<Vec<u8>, BencodeDecodedValue> = HashMap::new();

        let interval_decoded: BencodeDecodedValue =
            BencodeDecodedValue::Integer(response.interval_in_seconds as i64);
        let tracker_id_decoded: BencodeDecodedValue =
            BencodeDecodedValue::String(response.tracker_id.as_bytes().to_vec());
        let complete_decoded: BencodeDecodedValue =
            BencodeDecodedValue::Integer(response.complete as i64);
        let incomplete_decoded: BencodeDecodedValue =
            BencodeDecodedValue::Integer(response.incomplete as i64);

        let mut benencoded_peers: Vec<BencodeDecodedValue> = Vec::new();
        for peer in response.peers {
            let mut peer_map: HashMap<Vec<u8>, BencodeDecodedValue> = HashMap::new();
            peer_map.insert(
                "peer_id".as_bytes().to_vec(),
                BencodeDecodedValue::String(peer.peer_id),
            );
            peer_map.insert(
                "ip".as_bytes().to_vec(),
                BencodeDecodedValue::String(peer.ip.as_bytes().to_vec()),
            );
            peer_map.insert(
                "port".as_bytes().to_vec(),
                BencodeDecodedValue::Integer(peer.port as i64),
            );
            benencoded_peers.push(BencodeDecodedValue::Dictionary(peer_map));
        }
        let peers_decoded: BencodeDecodedValue = BencodeDecodedValue::List(benencoded_peers);

        response_map.insert("interval".as_bytes().to_vec(), interval_decoded);
        response_map.insert("tracker_id".as_bytes().to_vec(), tracker_id_decoded);
        response_map.insert("complete".as_bytes().to_vec(), complete_decoded);
        response_map.insert("incomplete".as_bytes().to_vec(), incomplete_decoded);
        response_map.insert("peers".as_bytes().to_vec(), peers_decoded);

        let response_decoded: BencodeDecodedValue = BencodeDecodedValue::Dictionary(response_map);
        encode(&response_decoded)
    }
}
