use log::*;
use rand::Rng;
use std::fmt;
use std::fmt::Display;
// use all the modules config, peer, tracker, metainfo
use crate::config::{Config, ConfigError};
use crate::metainfo::{Metainfo, MetainfoParserError};
use crate::peer::Peer;
use crate::tracker::{TrackerError, TrackerService};

const CONFIG_PATH: &str = "config.txt";

use std::io::{Read, Write};
use std::net::TcpStream;

// create_handsake_message creates the handshake message as a vector of u8
fn create_handshake_message(info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
    let mut handshake_message = Vec::new();
    handshake_message.extend_from_slice(&[19]);
    handshake_message.extend_from_slice(b"BitTorrent protocol");
    handshake_message.extend_from_slice(&[0u8; 8]);
    handshake_message.extend_from_slice(info_hash);
    handshake_message.extend_from_slice(peer_id);
    handshake_message
}

fn download_from_peer(peer: &Peer, client_peer_id: &[u8], info_hash: &[u8]) {
    let mut stream = TcpStream::connect(format!("{}:{}", peer.ip, peer.port)).unwrap();
    let handshake_message = create_handshake_message(info_hash, client_peer_id);
    let handshake_message = handshake_message.as_slice();
    stream.write_all(handshake_message).unwrap();
    // read exactly 68 bytes from stread and save the bytes in res
    let mut res = [0u8; 68];
    stream.read_exact(&mut res).unwrap();
    debug!("handshake msg from peer: {:?}", res);
}

pub fn run_with_torrent(torrent_path: &str) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    info!("Starting bittorrent client...");
    let peer_id = rand::thread_rng().gen::<[u8; 20]>();
    let config = Config::from_path(CONFIG_PATH)?;
    let metainfo = Metainfo::from_torrent(torrent_path)?;
    let tracker_service = TrackerService::from_metainfo(&metainfo, config.listen_port, &peer_id);
    let tracker_response = tracker_service.get_peers()?;
    if let Some(peer) = tracker_response.peers.get(0) {
        download_from_peer(peer, &peer_id, &metainfo.info_hash);
    }
    info!("Exited Bitorrent client successfully!");
    Ok(())
}

#[derive(Debug)]
/// The error type that is returned by the application
pub enum ApplicationError {
    ConfigError(ConfigError),
    MetainfoError(MetainfoParserError),
    TrackerError(TrackerError),
}

impl From<ConfigError> for ApplicationError {
    fn from(error: ConfigError) -> Self {
        ApplicationError::ConfigError(error)
    }
}

impl From<MetainfoParserError> for ApplicationError {
    fn from(error: MetainfoParserError) -> Self {
        ApplicationError::MetainfoError(error)
    }
}

impl From<TrackerError> for ApplicationError {
    fn from(error: TrackerError) -> Self {
        ApplicationError::TrackerError(error)
    }
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::ConfigError(error) => write!(f, "Config Error - {}", error),
            ApplicationError::MetainfoError(error) => write!(f, "Metainfo Error - {}", error),
            ApplicationError::TrackerError(error) => write!(f, "Tracker Error - {}", error),
        }
    }
}
