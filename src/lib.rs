pub mod bencode;
pub mod config;
pub mod metainfo;
pub mod tracker;
use config::Config;
use log::*;
use metainfo::Metainfo;
use rand::Rng;
use std::fmt;
use std::fmt::Display;
use tracker::*;

const CONFIG_PATH: &str = "config.txt";

pub fn run_with_torrent(torrent_path: &str) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    info!("Starting bittorrent client...");
    let peer_id = rand::thread_rng().gen::<[u8; 20]>();
    let config = Config::from_path(CONFIG_PATH)?;
    let metainfo = Metainfo::from_torrent(torrent_path)?;
    let tracker_service = TrackerService::from_metainfo(metainfo, config.listen_port, &peer_id);
    let peer_list = tracker_service.get_peers()?;
    info!("{:?}", peer_list);
    info!("Exited Bitorrent client successfully");
    Ok(())
}

#[derive(Debug)]
/// The error type that is returned by the application
pub enum ApplicationError {
    ConfigError(config::ConfigError),
    MetainfoError(metainfo::MetainfoParserError),
    TrackerError(tracker::TrackerError),
}

impl From<config::ConfigError> for ApplicationError {
    fn from(error: config::ConfigError) -> Self {
        ApplicationError::ConfigError(error)
    }
}

impl From<metainfo::MetainfoParserError> for ApplicationError {
    fn from(error: metainfo::MetainfoParserError) -> Self {
        ApplicationError::MetainfoError(error)
    }
}

impl From<tracker::TrackerError> for ApplicationError {
    fn from(error: tracker::TrackerError) -> Self {
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
