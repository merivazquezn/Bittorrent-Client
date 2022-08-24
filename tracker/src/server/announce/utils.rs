use super::{AnnounceRequest, TrackerEvent};
use crate::server::announce::constants::*;
use crate::server::announce::TrackerResponse;
use crate::server::errors::AnnounceError;
use bittorrent_rustico::bencode::encode;
use bittorrent_rustico::bencode::BencodeDecodedValue;
use chrono::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;

/// Parses the peer announce request
/// Receives the HTTP request query params
/// It demands default values, and if they are not present, it will return an error
/// It will also replace optional values such as events acordingly
///
/// # Returns:
///
/// ## On Success:
/// Returns the parsed announce request
///
/// ## On Error:
/// Returns an error if the request is not valid
/// Or if any of the required fields are not present
pub fn parse_request_from_params(
    params: HashMap<String, String>,
    address: SocketAddr,
) -> Result<AnnounceRequest, AnnounceError> {
    let missing_params: Vec<String> = get_missing_mandatory_params(&params);
    if !missing_params.is_empty() {
        return Err(AnnounceError::MissingParam(missing_params.join(", ")));
    }

    let info_hash: Vec<u8> = params.get(INFO_HASH_KEY).unwrap().clone().into_bytes();
    let peer_id: Vec<u8> = params.get(PEER_ID_KEY).unwrap().clone().into_bytes();
    let uploaded: u32 = parse_entry_to_u32(&params, UPLOADED_KEY)?;
    let downloaded: u32 = parse_entry_to_u32(&params, DOWNLOADED_KEY)?;
    let left: u32 = parse_entry_to_u32(&params, LEFT_KEY)?;
    let listening_port: u32 = parse_entry_to_u32(&params, PORT_KEY)?;

    let mut event: TrackerEvent = TrackerEvent::KeepAlive;
    if params.contains_key("event") {
        let tmp: String = params.get("event").unwrap().to_string();
        if tmp == "started" {
            event = TrackerEvent::Started;
        } else if tmp == "stoped" {
            event = TrackerEvent::Stopped;
        } else if tmp == "completed" {
            event = TrackerEvent::Completed;
        } else {
            return Err(AnnounceError::BadRequest);
        }
    }

    let mut numwant: u32 = DEFAULT_NUMWANT;
    if params.contains_key("numwant") {
        numwant = parse_entry_to_u32(&params, "numwant")?;
    }

    Ok(AnnounceRequest {
        info_hash,
        peer_id,
        port: listening_port as u16,
        uploaded,
        downloaded,
        left,
        event,
        numwant,
        ip: address.ip().to_string(),
    })
}

fn parse_entry_to_u32(params: &HashMap<String, String>, key: &str) -> Result<u32, AnnounceError> {
    params
        .get(key)
        .unwrap()
        .parse()
        .map_err(|_| AnnounceError::BadRequest)
}

fn get_missing_mandatory_params(params: &HashMap<String, String>) -> Vec<String> {
    let mut missing_params: Vec<String> = Vec::new();
    let mandatory_params: Vec<String> = vec![
        INFO_HASH_KEY,
        PEER_ID_KEY,
        UPLOADED_KEY,
        DOWNLOADED_KEY,
        LEFT_KEY,
        PORT_KEY,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

    for param in mandatory_params {
        if !params.contains_key(&param) {
            missing_params.push(param.to_string());
        }
    }

    missing_params
}

/// Decides whether the peer is active or not, according to the last_announce timestamp
/// It is not active if it has passed more than 2 times the tracker interval
pub fn is_active_peer(last_announce: DateTime<Local>, interval: u32) -> bool {
    let time_between_announces: chrono::Duration = Local::now() - last_announce;
    time_between_announces < chrono::Duration::seconds((2 * interval).into())
}

/// Decides if a peer request says that it is a seeder or not
pub fn has_completed(request: &AnnounceRequest) -> bool {
    request.event == TrackerEvent::Completed
}

/// Decides if a peer request says that it is stopping their server
pub fn is_peer_stopping(request: &AnnounceRequest) -> bool {
    request.event == TrackerEvent::Stopped
}

/// It encodes the tracker response and return the bytes of the response
/// It is encoded with bencoding encoding.
pub fn get_response_bytes(response: TrackerResponse) -> Vec<u8> {
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
            PEER_ID_KEY.as_bytes().to_vec(),
            BencodeDecodedValue::String(peer.peer_id),
        );
        peer_map.insert(
            "ip".as_bytes().to_vec(),
            BencodeDecodedValue::String(peer.ip.as_bytes().to_vec()),
        );
        peer_map.insert(
            PORT_KEY.as_bytes().to_vec(),
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
