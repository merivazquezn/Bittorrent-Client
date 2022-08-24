use rand::Rng;
use sha1::{Digest, Sha1};

pub fn generate_peer_id() -> [u8; 20] {
    rand::thread_rng().gen::<[u8; 20]>()
}

// generate peer id, a 20 byte string hashing the config path
pub fn generate_peer_id_from_config_path(_config: &str) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(std::env::var("INDEX").unwrap_or_else(|_| "".to_string()));
    let peer_id = hasher.finalize().to_vec();
    let mut result = [0u8; 20];
    result[..20].clone_from_slice(&peer_id[..20]);
    println!("{:?}", result);
    result
}
