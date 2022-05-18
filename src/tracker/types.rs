pub enum Event {
    Started,
    Completed,
    Stopped,
}

pub struct RequestParameters {
    pub info_hash: Vec<u8>,
    pub peer_id: Vec<u8>,
    pub port: u32,
    pub uploaded: u32,
    pub downloaded: u32,
    pub left: u32,
    pub event: Event,
}

#[derive(Debug)]
pub struct Peer {
    pub ip: String,
    pub port: i64,
    pub peer_id: Vec<u8>,
}

#[derive(Debug)]
pub struct TrackerResponse {
    pub peers: Vec<Peer>,
}
