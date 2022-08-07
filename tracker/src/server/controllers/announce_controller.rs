use crate::http::HttpError;
use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::server::announce::parse_request_from_params;
use crate::server::announce::AnnounceManager;
use crate::server::announce::AnnounceRequest;
use crate::server::announce::TrackerResponse;
use crate::server::errors::AnnounceError;
use bittorrent_rustico::bencode::encode;
use bittorrent_rustico::bencode::BencodeDecodedValue;
use std::collections::HashMap;

pub struct AnnounceController;

impl AnnounceController {
    pub fn handle_announce(
        http_service: Box<dyn IHttpService>,
        request: HttpGetRequest,
        announce_manager: AnnounceManager,
    ) -> Result<(), AnnounceError> {
        let params: HashMap<String, String> = request.params;
        let announce_request: AnnounceRequest =
            parse_request_from_params(params, http_service.get_client_address())?;
        let response: TrackerResponse =
            announce_manager.announce_and_get_response(announce_request)?;

        Self::send_response(http_service, response)?;
        Ok(())
    }

    fn send_response(
        mut http_service: Box<dyn IHttpService>,
        response: TrackerResponse,
    ) -> Result<(), HttpError> {
        let response_bytes: Vec<u8> = Self::get_response_bytes(response);
        http_service.send_ok_response(response_bytes, "application/octet-stream".to_string())
    }

    /// Encodes with bencoding the tracker response, and returns the bytes to be sent
    fn get_response_bytes(response: TrackerResponse) -> Vec<u8> {
        let mut response_map: HashMap<Vec<u8>, BencodeDecodedValue> = HashMap::new();

        let interval_decoded: BencodeDecodedValue =
            BencodeDecodedValue::Integer(response.interval_in_seconds as i64);
        let tracker_id_decoded: BencodeDecodedValue =
            BencodeDecodedValue::String(response.tracker_id.as_bytes().to_vec());
        let complete_decoded: BencodeDecodedValue =
            BencodeDecodedValue::Integer(response.complete as i64);
        let incomplete_decoded: BencodeDecodedValue =
            BencodeDecodedValue::Integer(response.incomplete as i64);

        let mut benencoded_peers: Vec<BencodeDecodedValue> = Vec::new();
        for peer in response.peers {
            let mut peer_map: HashMap<Vec<u8>, BencodeDecodedValue> = HashMap::new();
            peer_map.insert(
                "peer_id".as_bytes().to_vec(),
                BencodeDecodedValue::String(peer.peer_id),
            );
            peer_map.insert(
                "ip".as_bytes().to_vec(),
                BencodeDecodedValue::String(peer.ip.as_bytes().to_vec()),
            );
            peer_map.insert(
                "port".as_bytes().to_vec(),
                BencodeDecodedValue::Integer(peer.port as i64),
            );
            benencoded_peers.push(BencodeDecodedValue::Dictionary(peer_map));
        }
        let peers_decoded: BencodeDecodedValue = BencodeDecodedValue::List(benencoded_peers);

        response_map.insert("interval".as_bytes().to_vec(), interval_decoded);
        response_map.insert("tracker_id".as_bytes().to_vec(), tracker_id_decoded);
        response_map.insert("complete".as_bytes().to_vec(), complete_decoded);
        response_map.insert("incomplete".as_bytes().to_vec(), incomplete_decoded);
        response_map.insert("peers".as_bytes().to_vec(), peers_decoded);

        let response_decoded: BencodeDecodedValue = BencodeDecodedValue::Dictionary(response_map);
        encode(&response_decoded)
    }
}
