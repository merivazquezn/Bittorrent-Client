use crate::peer::Peer;
use std::time::Duration;

#[derive(PartialEq)]
pub enum Event {
    Started,
    Completed,
    Stopped,
    KeepAlive,
}

impl Event {
    pub fn as_string(&self) -> String {
        match self {
            Event::Started => "started".to_string(),
            Event::Completed => "completed".to_string(),
            Event::Stopped => "stopped".to_string(),
            Event::KeepAlive => "".to_string(),
        }
    }
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
    pub interval: Option<Duration>,
}
