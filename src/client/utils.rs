use rand::Rng;

pub fn generate_peer_id() -> [u8; 20] {
    rand::thread_rng().gen::<[u8; 20]>()
}
