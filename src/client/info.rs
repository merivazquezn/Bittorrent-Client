use super::constants::*;
use super::utils::generate_peer_id;
use crate::application_errors::ApplicationError;
use crate::config::Config;
use crate::metainfo::Metainfo;

pub struct ClientInfo {
    pub peer_id: [u8; 20],
    pub config: Config,
    pub metainfo: Metainfo,
}

impl ClientInfo {
    pub fn new(torrent_path: &str, config_path: &str) -> Result<ClientInfo, ApplicationError> {
        let config = Config::from_path(config_path)?;
        let peer_id = generate_peer_id();
        let metainfo = Metainfo::from_torrent(torrent_path)?;

        Ok(ClientInfo {
            config,
            peer_id,
            metainfo,
        })
    }
}
