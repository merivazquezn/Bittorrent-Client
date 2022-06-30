use super::constants::*;
use super::errors::TrackerError;
use super::types::RequestParameters;
use super::types::TrackerResponse;
use super::types::*;
use super::utils::*;
use crate::bencode::BencodeDecodedValue;
use crate::bencode::*;
use crate::http::IHttpService;
use crate::metainfo::Metainfo;
use crate::peer::peer_message_service_provider;
use crate::peer::Peer;
use log::*;
use rand::Rng;

pub struct TrackerService {
    request_parameters: RequestParameters,
    http_service: Box<dyn IHttpService>,
}

impl TrackerService {
    pub fn from_metainfo(
        metainfo: &Metainfo,
        listen_port: u16,
        peer_id: &[u8; 20],
        http_service: Box<dyn IHttpService>,
    ) -> TrackerService {
        debug!("Parsing tracker request parameters");
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
            http_service,
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
    /// ```no_run
    /// use bittorrent_rustico::tracker::TrackerService;
    /// use bittorrent_rustico::config::Config;
    /// use bittorrent_rustico::metainfo::Metainfo;
    /// use bittorrent_rustico::http::HttpsService;
    /// use rand::Rng;
    ///
    /// const CONFIG_PATH: &str = "config.txt";
    /// let torrent_path = "ubuntu.torrent";
    /// let peer_id = rand::thread_rng().gen::<[u8; 20]>();
    /// let config = Config::from_path(CONFIG_PATH).unwrap();
    /// let metainfo = Metainfo::from_torrent(torrent_path).unwrap();
    /// let mut http_service = HttpsService::from_url(&metainfo.announce).unwrap();
    /// let mut tracker_service = TrackerService::from_metainfo(&metainfo, config.listen_port, &peer_id, Box::new(http_service));
    /// let peer_list = tracker_service.get_peers().unwrap();
    /// println!("{:?}", peer_list);
    /// ```
    pub fn get_peers(&mut self) -> Result<TrackerResponse, TrackerError> {
        debug!("Sending tracker get request");
        let response: Vec<u8> = self.http_service.get(
            "/announce",
            &parameters_to_querystring(&self.request_parameters),
        )?;
        debug!("parsing tracker response");
        match self.parse_response(decode(&response)?) {
            Ok(tracker_response) => Ok(tracker_response),
            Err(err) => Err(err),
        }
    }

    // Builds the TrackerResponse from the bencoded data
    fn parse_response(
        &self,
        bencoded_response: BencodeDecodedValue,
    ) -> Result<TrackerResponse, TrackerError> {
        let response_dic = bencoded_response.get_as_dictionary()?;
        trace!("Parsing peer list from response");
        match response_dic.get(PEERS) {
            Some(BencodeDecodedValue::List(peer_list)) => {
                let peer_list = self.build_peer_list(peer_list)?;
                Ok(TrackerResponse { peers: peer_list })
            }
            Some(BencodeDecodedValue::String(peer_list)) => {
                let peer_list = self.build_peer_list_from_binary(peer_list)?;
                Ok(TrackerResponse { peers: peer_list })
            }
            Some(_) => Err(TrackerError::InvalidResponse(
                "Peer list was neither a list or a compact string".to_string(),
            )),
            None => {
                let error_message = response_dic
                    .get(&FAILURE_REASON.to_vec())
                    .ok_or_else(|| {
                        TrackerError::InvalidResponse("request failed with no reason".to_string())
                    })?
                    .get_as_string()?;
                let error_message = u8_to_string(error_message).ok_or_else(|| {
                    TrackerError::InvalidResponse(
                        "request failed and returned non utf8 reason".to_string(),
                    )
                })?;
                Err(TrackerError::InvalidResponse(error_message))
            }
        }
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
                None => {
                    return Err(TrackerError::InvalidResponse(format!(
                        "missing ip of peer {:?}",
                        peer_dic
                    )))
                }
            };
            let port = match peer_dic.get(PORT) {
                // parse the port as a u16
                Some(port) => *port.get_as_integer()? as u16,
                None => {
                    return Err(TrackerError::InvalidResponse(format!(
                        "missing port of peer {:?}",
                        peer_dic
                    )))
                }
            };
            let peer_id = match peer_dic.get(PEER_ID) {
                Some(peer_id) => peer_id.get_as_string()?.to_vec(),
                None => {
                    // we create a random peer id if they don't provide one
                    rand::thread_rng().gen::<[u8; 20]>().to_vec()
                }
            };

            let peer = Peer {
                ip: u8_to_string(peer_ip).ok_or_else(|| {
                    TrackerError::InvalidResponse(format!("invalid peer ip: {:?}", peer_id))
                })?,
                port,
                peer_id,
                peer_message_service_provider,
            };

            peer_list.push(peer);
        }

        Ok(peer_list)
    }

    // given a bencoded_peer_list, which is a byte slice, it creates a vector of Peers, which are structs with ip, port, and peer_id.
    // to do that, we go through the entire bencoded_peer_list, taking the first 4 bytes to form the IP and the next 2 bytes to form the port.
    // we do that for every peer until we reach the end of the bencoded_peer_list.
    // the peer_id is random
    // to join the 4 bytes of the ip, we convert them into the form "xxx.xxx.xxx.xxx"
    fn build_peer_list_from_binary(
        &self,
        bencoded_peer_list: &[u8],
    ) -> Result<Vec<Peer>, TrackerError> {
        let mut peer_list: Vec<Peer> = Vec::new();
        let mut i = 0;
        while i < bencoded_peer_list.len() {
            let ip = &bencoded_peer_list[i..i + 4];
            let port = &bencoded_peer_list[i + 4..i + 6];
            let peer = Peer {
                ip: self.convert_4_bytes_to_ip_string(ip),
                port: u16::from_be_bytes([port[0], port[1]]),
                peer_id: rand::thread_rng().gen::<[u8; 20]>().to_vec(),
                peer_message_service_provider,
            };
            peer_list.push(peer);
            i += 6;
        }

        Ok(peer_list)
    }

    fn convert_4_bytes_to_ip_string(&self, ip_bytes: &[u8]) -> String {
        let mut ip_string = String::new();
        for i in ip_bytes.iter().take(4) {
            ip_string.push_str(&format!("{}.", *i));
        }
        ip_string.pop();
        ip_string
    }
}

// tests
#[cfg(test)]

mod tests {
    use super::*;
    use crate::bencode;
    use crate::config::Config;
    use crate::http::MockHttpsService;
    use crate::metainfo::Metainfo;
    use rand::Rng;
    use std::collections::HashMap;

    #[test]
    fn test_get_peers_failure_on_not_read_bytes() {
        const CONFIG_PATH: &str = "config.txt";
        let torrent_path = "ubuntu.torrent";
        let peer_id = rand::thread_rng().gen::<[u8; 20]>();
        let config = Config::from_path(CONFIG_PATH).expect("Failed to load config");
        let metainfo = Metainfo::from_torrent(torrent_path).expect("Failed to load metainfo");
        let connection = Box::new(MockHttpsService { read_bytes: vec![] });
        let mut tracker_service =
            TrackerService::from_metainfo(&metainfo, config.listen_port, &peer_id, connection);
        let result = tracker_service.get_peers();
        println!("result {:?}", result);
        assert!(matches!(
            tracker_service.get_peers(),
            Err(TrackerError::BencodeError(_))
        ));
    }

    #[test]
    fn test_get_peers_success_on_valid_response_containing_one_peer() {
        const CONFIG_PATH: &str = "config.txt";
        let torrent_path = "ubuntu.torrent";
        let peer_id = rand::thread_rng().gen::<[u8; 20]>();
        let config = Config::from_path(CONFIG_PATH).unwrap();
        let metainfo = Metainfo::from_torrent(torrent_path).unwrap();
        let bencoded_response = BencodeDecodedValue::Dictionary(HashMap::from([(
            PEERS.to_vec(),
            BencodeDecodedValue::List(vec![BencodeDecodedValue::Dictionary(HashMap::from([
                (
                    IP.to_vec(),
                    BencodeDecodedValue::String(b"0.0.0.0".to_vec()),
                ),
                (PORT.to_vec(), BencodeDecodedValue::Integer(10000)),
                (
                    PEER_ID.to_vec(),
                    BencodeDecodedValue::String([0u8; 20].to_vec()),
                ),
            ]))]),
        )]));

        let connection = Box::new(MockHttpsService {
            read_bytes: bencode::encode(&bencoded_response),
        });
        let mut tracker_service =
            TrackerService::from_metainfo(&metainfo, config.listen_port, &peer_id, connection);
        assert_eq!(tracker_service.get_peers().unwrap().peers.len(), 1);
        assert_eq!(
            tracker_service.get_peers().unwrap().peers[0],
            Peer {
                ip: "0.0.0.0".to_string(),
                port: 10000,
                peer_id: [0u8; 20].to_vec(),
                peer_message_service_provider
            }
        );
    }
}
