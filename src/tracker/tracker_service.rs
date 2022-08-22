use super::constants::*;
use super::errors::TrackerError;
use super::types::RequestParameters;
use super::types::TrackerResponse;
use super::types::*;
use super::utils::*;
use crate::bencode::BencodeDecodedValue;
use crate::bencode::*;
use crate::client::ClientInfo;
use crate::download_manager::get_existing_pieces;
use crate::http::HttpsService;
use crate::http::IHttpService;
use crate::peer::peer_message_service_provider;
use crate::peer::Peer;
use log::*;
use rand::Rng;
use std::collections::HashMap;
use std::time::Duration;

pub trait ITrackerService: Clone {
    fn announce(&mut self, event: Option<Event>) -> Result<TrackerResponse, TrackerError>;
}

#[derive(Clone)]
pub struct TrackerService {
    client_info: ClientInfo,
}

impl TrackerService {
    pub fn new(client_info: ClientInfo) -> Self {
        TrackerService { client_info }
    }

    fn parse_response(
        &self,
        bencoded_response: BencodeDecodedValue,
    ) -> Result<TrackerResponse, TrackerError> {
        let response_dic = bencoded_response.get_as_dictionary()?;
        trace!("Parsing peer list from response");

        let peers = self.get_peers_from_response(response_dic)?;
        match self.get_min_interval_from_response(bencoded_response) {
            Ok(interval_rec) => Ok(TrackerResponse {
                peers,
                interval: Some(interval_rec),
            }),
            Err(_) => Ok(TrackerResponse {
                peers,
                interval: None,
            }),
        }
    }

    fn get_peers_from_response(
        &self,
        response_dic: &HashMap<Vec<u8>, BencodeDecodedValue>,
    ) -> Result<Vec<Peer>, TrackerError> {
        match response_dic.get(PEERS) {
            Some(BencodeDecodedValue::List(peer_list)) => {
                let peer_list = self.build_peer_list(peer_list)?;
                Ok(peer_list)
            }
            Some(BencodeDecodedValue::String(peer_list)) => {
                let peer_list = self.build_peer_list_from_binary(peer_list)?;
                Ok(peer_list)
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

    fn get_min_interval_from_response(
        &self,
        response: BencodeDecodedValue,
    ) -> Result<Duration, TrackerError> {
        let response_dic = response.get_as_dictionary()?;
        let interval = response_dic
            .get(INTERVAL)
            .ok_or_else(|| TrackerError::InvalidResponse("interval not found".to_string()))?
            .get_as_integer()?;

        Ok(Duration::from_secs(*interval as u64))
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
            println!("{:?}", peer_dic.get(PORT));
            let port = match peer_dic.get(PORT) {
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

impl ITrackerService for TrackerService {
    fn announce(&mut self, event: Option<Event>) -> Result<TrackerResponse, TrackerError> {
        debug!("Sending tracker announce request");
        let mut http_service = HttpsService::from_url(&self.client_info.metainfo.announce)?;
        let pieces_dir = format!(
            "{}/{}/pieces",
            self.client_info.config.download_path, self.client_info.metainfo.info.name
        );
        let initial_pieces: Vec<u32> = get_existing_pieces(
            self.client_info.metainfo.get_piece_count(),
            pieces_dir.as_str(),
        );
        let downloaded = if initial_pieces.len() as u32
            * self.client_info.metainfo.info.piece_length as u32
            > self.client_info.metainfo.info.length as u32
        {
            self.client_info.metainfo.info.length as u32
        } else {
            initial_pieces.len() as u32 * self.client_info.metainfo.info.piece_length as u32
        };

        let left = self.client_info.metainfo.info.length as u32 - downloaded;

        let request_parameters = RequestParameters {
            info_hash: self.client_info.metainfo.info_hash.to_vec(),
            peer_id: self.client_info.peer_id.to_vec(),
            port: self.client_info.config.listen_port,
            uploaded: 0,
            downloaded,
            left,
            event: event.unwrap_or(Event::KeepAlive),
        };

        let response: Vec<u8> =
            http_service.get("/announce", &parameters_to_querystring(&request_parameters))?;
        debug!("parsing tracker response");

        match self.parse_response(decode(&response)?) {
            Ok(tracker_response) => Ok(tracker_response),
            Err(err) => Err(err),
        }
    }
}

#[derive(Clone)]
pub struct MockTrackerService {
    pub responses: Vec<Vec<Peer>>,
    pub response_index: usize,
}

impl ITrackerService for MockTrackerService {
    fn announce(&mut self, _: Option<Event>) -> Result<TrackerResponse, TrackerError> {
        if self.response_index < self.responses.len() {
            Ok(TrackerResponse {
                peers: self.responses[self.response_index].clone(),
                interval: None,
            })
        } else {
            Err(TrackerError::InvalidResponse("request failed".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::metainfo::Metainfo;
    use rand::Rng;

    #[test]
    fn test_get_peers_failure_on_invalid_or_not_found_response() {
        const CONFIG_PATH: &str = "src/config/test_files/correct_config.txt";
        let torrent_path = "./example_torrents/ubuntu.torrent";
        let peer_id = rand::thread_rng().gen::<[u8; 20]>();
        let config = Config::from_path(CONFIG_PATH).expect("Failed to load config");
        let metainfo = Metainfo::from_torrent(torrent_path).expect("Failed to load metainfo");

        let mut tracker_service = TrackerService::new(ClientInfo {
            peer_id,
            config,
            metainfo,
        });

        let response = tracker_service.announce(None);
        println!("{:?}", response);
        assert!(matches!(response, Err(TrackerError::InvalidResponse(_))));
    }
}
