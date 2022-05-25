use crate::peer::Peer;

#[allow(dead_code)]
pub enum Event {
    Started,
    Completed,
    Stopped,
}

pub struct RequestParameters {
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
    pub port: u16,
    pub uploaded: u32,
    pub downloaded: u32,
    pub left: u32,
    pub event: Event,
}

#[derive(Debug, PartialEq)]
pub struct TrackerResponse {
    pub peers: Vec<Peer>,
}
