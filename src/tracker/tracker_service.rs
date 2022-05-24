use super::constants::*;
use super::errors::TrackerError;
use super::types::RequestParameters;
use super::types::TrackerResponse;
use super::types::*;
use super::utils::*;
use crate::bencode::BencodeDecodedValue;
use crate::bencode::*;
use crate::metainfo::Metainfo;
use crate::peer::Peer;
use crate::tcp_connection::TcpConnection;

pub struct TrackerService {
    request_parameters: RequestParameters,
    connection: Box<dyn TcpConnection>,
}

impl TrackerService {
    pub fn from_metainfo(
        metainfo: &Metainfo,
        listen_port: u16,
        peer_id: &[u8; 20],
        connection: Box<dyn TcpConnection>,
    ) -> TrackerService {
        TrackerService {
            request_parameters: RequestParameters {
                info_hash: metainfo.info_hash.clone(),
                peer_id: peer_id.to_vec(),
                port: listen_port,
                // for downloading once, it's ok to set it to 0
                uploaded: 0,
                downloaded: 0,
                left: 0,
                event: Event::Started,
            },
            connection,
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
    ///
    /// ```ignore
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
    /// let tracker_service = TrackerService::from_metainfo(&metainfo, config.listen_port, &peer_id);
    /// let peer_list = tracker_service.get_peers().unwrap();
    /// println!("{:?}", peer_list);
    /// ```
    ///
    pub fn get_peers(&mut self) -> Result<TrackerResponse, TrackerError> {
        let mut request = String::new();

        request.push_str(&format!(
            "GET {}?{} HTTP/1.0\r\n",
            "/announce",
            parameters_to_querystring(&self.request_parameters)
        ));
        request.push_str("Host: torrent.ubuntu.com");
        request.push_str("\r\n\r\n");

        self.connection.write(request.as_bytes())?;
        let mut res: Vec<u8> = Vec::new();
        self.connection.read(&mut res)?;

        let bytes_after_rn = bencode_response(&res);
        let decoded: BencodeDecodedValue = decode(&bytes_after_rn)?;
        match self.parse_response(decoded) {
            Ok(response) => Ok(response),
            Err(error) => Err(error),
        }
    }

    // Builds the TrackerResponse from the bencoded data
    fn parse_response(
        &self,
        bencoded_response: BencodeDecodedValue,
    ) -> Result<TrackerResponse, TrackerError> {
        let response_dic = bencoded_response.get_as_dictionary()?;
        let benencoded_peers_list = match response_dic.get(PEERS) {
            Some(peers) => peers.get_as_list()?,
            None => {
                let error_message = response_dic
                    .get(&FAILURE_REASON.to_vec())
                    .ok_or(TrackerError::InvalidResponse)?
                    .get_as_string()?;
                let error_message = u8_to_string(error_message);
                return Err(TrackerError::ResponseError(error_message));
            }
        };

        let peer_list = self.build_peer_list(benencoded_peers_list)?;
        Ok(TrackerResponse { peers: peer_list })
    }

    fn build_peer_list(
        &self,
        bencoded_peer_list: &[BencodeDecodedValue],
    ) -> Result<Vec<Peer>, TrackerError> {
        let mut peer_list: Vec<Peer> = Vec::new();

        for value in bencoded_peer_list.iter() {
            let peer_dic = value.get_as_dictionary()?;
            let peer_ip = match peer_dic.get(IP) {
                Some(ip) => ip.get_as_string()?,
                None => return Err(TrackerError::InvalidResponse),
            };
            let peer_port = match peer_dic.get(PORT) {
                // parse the port as a u16
                Some(port) => *port.get_as_integer()? as u16,
                None => return Err(TrackerError::InvalidResponse),
            };
            let peer_id = match peer_dic.get(PEER_ID) {
                Some(peer_id) => peer_id.get_as_string()?,
                None => return Err(TrackerError::InvalidResponse),
            };
            let peer = Peer {
                ip: u8_to_string(peer_ip),
                port: peer_port,
                peer_id: peer_id.to_vec(),
            };

            peer_list.push(peer);
        }

        Ok(peer_list)
    }
}
