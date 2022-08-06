use crate::http::IHttpService;
use chrono::prelude::*;

pub enum AnnounceMessage {
    Announce(AnnounceRequest, Box<dyn IHttpService>),
}

/// Event that identifies what is the peer's state
#[derive(PartialEq)]
pub enum TrackerEvent {
    /// Peer wants to start the download
    Started,
    /// Peer is leaving the torrent network
    Stopped,
    /// Peer has completed the download
    Completed,
    /// Peer sends a keep-alive message so that we mantain it inside the network
    KeepAlive,
}

/// Data collected from a announce request
pub struct AnnounceRequest {
    /// 20-bytes long vector that identifies the torrent
    pub info_hash: Vec<u8>,
    /// 20-bytes long vector representing the id of a peer
    pub peer_id: Vec<u8>,
    /// The port from which the peer is contacting the tracker
    pub port: u16,
    /// Event that identifies what is the peer's state
    pub event: TrackerEvent,
    /// Ip address of the peer
    pub ip: String,
    /// Amount of peers the client peer want to be given
    pub numwant: u32,
    /// The amount of bytes that the peer has shared with other peers
    pub uploaded: u32,
    /// The amount of bytes that the peer has downloaded from other peers
    pub downloaded: u32,
    /// The amount of bytes that the needs to download in order to complete the download
    pub left: u32,
}

#[derive(Clone)]
/// Represents the important data of a single peer to be sent to other peers
pub struct Peer {
    /// Peer's ip address
    pub ip: String,
    /// Peer's listen port
    pub port: u16,
    /// 20 bytes long vector representing peer's id
    pub peer_id: Vec<u8>,
}

/// Represents a peer in a certain torrent network
pub struct PeerEntry {
    /// Stores the data of the peer (ip, port and peer_id)
    pub peer: Peer,
    /// Timestamp representing the last time that the peer announced
    pub last_announce: DateTime<Local>,
    /// Whether the peer has or not downloaded the whole file
    pub is_seeder: bool,
}

/// Represents a list of peers in a certain torrent network
pub struct ActivePeers {
    /// The list of peers of the network. There may be inactive peers in the list
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
