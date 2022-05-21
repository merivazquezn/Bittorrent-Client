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

fn _download_from_peer(_peer: Peer) {}

pub fn run_with_torrent(torrent_path: &str) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    info!("Starting bittorrent client...");
    let peer_id = rand::thread_rng().gen::<[u8; 20]>();
    let config = Config::from_path(CONFIG_PATH)?;
    let metainfo = Metainfo::from_torrent(torrent_path)?;
    let tracker_service = TrackerService::from_metainfo(metainfo, config.listen_port, &peer_id);
    let _tracker_response = tracker_service.get_peers()?;
    info!("Exited Bitorrent client successfully");
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
