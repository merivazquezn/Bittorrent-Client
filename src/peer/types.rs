#[derive(Debug)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
    pub peer_id: Vec<u8>,
}
