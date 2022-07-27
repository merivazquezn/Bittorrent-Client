use crate::http::IHttpService;

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

pub struct Peer {
    pub ip: String,
    pub port: u16,
    pub peer_id: Vec<u8>,
}
