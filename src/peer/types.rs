#[derive(Debug)]
pub struct Peer {
    // TODO: move to peer module
    pub ip: String,
    pub port: i64,
    pub peer_id: Vec<u8>,
}
