use crate::http::IHttpService;
use chrono::prelude::*;

pub enum AnnounceMessage {
    Announce(AnnounceRequest, Box<dyn IHttpService>),
}

pub enum TrackerEvent {
    Started,
    Stopped,
    Completed,
    KeepAlive,
}

pub struct AnnounceRequest {
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
    pub port: u16,
    pub event: TrackerEvent,
    pub ip: String,
    pub numwant: u32,
    pub uploaded: u32,
    pub downloaded: u32,
    pub left: u32,
}

#[derive(Clone)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
    pub peer_id: Vec<u8>,
}

pub struct PeerEntry {
    pub peer: Peer,
    pub last_announce: DateTime<Local>, // this is a timestamp
}

pub struct ActivePeers {
    pub peers: Vec<PeerEntry>,
}

/// Represents the mandatory values of the tracker response
pub struct TrackerResponse {
    // Expected interval in seconds for keep_alive requests from other peers
    pub interval_in_seconds: u32,
    // Can be a random string
    pub tracker_id: String,
    /// Number peers with the entire file (seeders)
    pub complete: u32,
    /// Number of non-seeders peers (leechers)
    pub incomplete: u32,
    /// List of peers to send to the announced peer
    pub peers: Vec<Peer>,
}
