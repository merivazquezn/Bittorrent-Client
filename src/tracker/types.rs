use super::connection;
use super::errors::TrackerError;
use crate::metainfo::Metainfo;

pub enum Event {
    Started,
    Completed,
    Stopped,
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

#[derive(Debug)]
pub struct Peer {
    // TODO: move to peer module
    pub ip: String,
    pub port: i64,
    pub peer_id: Vec<u8>,
}

#[derive(Debug)]
pub struct TrackerResponse {
    pub peers: Vec<Peer>,
}

pub struct TrackerService {
    request_parameters: RequestParameters,
}

impl TrackerService {
    pub fn from_metainfo(
        metainfo: Metainfo,
        listen_port: u16,
        peer_id: &[u8; 20],
    ) -> TrackerService {
        TrackerService {
            request_parameters: RequestParameters {
                info_hash: metainfo.info_hash,
                peer_id: peer_id.to_vec(),
                port: listen_port,
                // for downloading once, it's ok to set it to 0
                uploaded: 0,
                downloaded: 0,
                left: 0,
                event: Event::Started,
            },
        }
    }

    /// Obtains peer list from the tracker
    ///
    /// Receives a [`RequestParameters`] struct with the necessary information to make the request
    ///
    /// Returns a Result holding:
    ///
    /// ## On succes
    /// - [`TrackerResponse`] struct with the peer list and the parsed tracker response
    ///
    /// ## On error
    /// - [`TrackerError`] struct with the error type and message
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use bittorrent_rustico::tracker::TrackerService;
    /// use bittorrent_rustico::config::Config;
    /// use bittorrent_rustico::metainfo::Metainfo;
    /// use rand::Rng;
    ///
    /// const CONFIG_PATH: &str = "config.txt";
    /// let torrent_path = "ubuntu.torrent";
    /// let peer_id = rand::thread_rng().gen::<[u8; 20]>();
    /// let config = Config::from_path(CONFIG_PATH).unwrap();
    /// let metainfo = Metainfo::from_torrent(torrent_path).unwrap();
    /// let tracker_service = TrackerService::from_metainfo(metainfo, config.listen_port, &peer_id);
    /// let peer_list = tracker_service.get_peers().unwrap();
    /// println!("{:?}", peer_list);
    /// ```
    ///
    pub fn get_peers(&self) -> Result<TrackerResponse, TrackerError> {
        connection::get_peer_list(&self.request_parameters)
    }
}
