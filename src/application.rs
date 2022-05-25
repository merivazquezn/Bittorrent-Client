use log::*;
use rand::Rng;
// use all the modules config, peer, tracker, metainfo
use crate::application_errors::ApplicationError;
use crate::config::Config;
use crate::http::HttpsConnection;
use crate::metainfo::Metainfo;
use crate::peer::Peer;
use crate::tracker::TrackerService;

const CONFIG_PATH: &str = "config.txt";
const PSTRLEN: u8 = 19;
const HANDSHAKE_LENGTH: usize = 68;

// Message constants
const MESSAGE_ID_SIZE: usize = 1;
const MESSAGE_LENGTH_SIZE: usize = 4;

use std::io::{Read, Write};
use std::net::TcpStream;

#[allow(dead_code)]
#[derive(Debug)]
struct PeerMessage {
    id: u8,
    length: u32,
    payload: Vec<u8>,
}

// TODO: Define error types
fn read_message_from_peer(
    stream: &mut TcpStream,
) -> Result<PeerMessage, Box<dyn std::error::Error>> {
    let mut message_length = [0u8; MESSAGE_LENGTH_SIZE];
    debug!("Reading message length");
    stream.read_exact(&mut message_length).unwrap();
    let message_length = u32::from_be_bytes(message_length);
    let mut message_id = [0u8; MESSAGE_ID_SIZE];
    debug!("Reading message id");
    stream.read_exact(&mut message_id).unwrap();
    let mut payload: Vec<u8> = vec![0; (message_length - 1) as usize];
    debug!("Reading message payload");
    stream.read_exact(&mut payload).unwrap();
    debug!("Message read");

    Ok(PeerMessage {
        id: message_id[0],
        length: message_length,
        payload,
    })
}

fn create_handshake_message(info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
    let mut handshake_message = Vec::new();
    handshake_message.extend_from_slice(&[PSTRLEN]);
    handshake_message.extend_from_slice(b"BitTorrent protocol");
    handshake_message.extend_from_slice(&[0u8; 8]);
    handshake_message.extend_from_slice(info_hash);
    handshake_message.extend_from_slice(peer_id);
    handshake_message
}

fn peer_communication(peer: &Peer, client_peer_id: &[u8], info_hash: &[u8]) {
    let mut stream = TcpStream::connect(format!("{}:{}", peer.ip, peer.port)).unwrap();
    let handshake_message = create_handshake_message(info_hash, client_peer_id);
    let handshake_message = &handshake_message;
    stream.write_all(handshake_message).unwrap();
    // read exactly 68 bytes from stread and save the bytes in res
    let mut res = [0u8; HANDSHAKE_LENGTH];
    stream.read_exact(&mut res).unwrap();
    let message = read_message_from_peer(&mut stream).unwrap();
    debug!("Message from peer: {:?}", message);
}

pub fn run_with_torrent(torrent_path: &str) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    info!("Starting bittorrent client...");
    let peer_id = rand::thread_rng().gen::<[u8; 20]>();
    let config = Config::from_path(CONFIG_PATH)?;
    let metainfo = Metainfo::from_torrent(torrent_path)?;
    let http_service = HttpsConnection::from_url(&metainfo.announce)?;
    let mut tracker_service = TrackerService::from_metainfo(
        &metainfo,
        config.listen_port,
        &peer_id,
        Box::new(http_service),
    );
    let tracker_response = tracker_service.get_peers()?;

    if let Some(peer) = tracker_response.peers.get(0) {
        peer_communication(peer, &peer_id, &metainfo.info_hash);
    }

    info!("Exited Bitorrent client successfully!");
    Ok(())
}
