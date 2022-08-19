use super::{AnnounceRequest, TrackerEvent};
use crate::server::announce::TrackerResponse;
use crate::server::errors::AnnounceError;
use bittorrent_rustico::bencode::encode;
use bittorrent_rustico::bencode::BencodeDecodedValue;
use chrono::prelude::*;
use std::collections::HashMap;
use std::net::SocketAddr;

const DEFAULT_NUMWANT: u32 = 50;

pub fn parse_request_from_params(
    params: HashMap<String, String>,
    address: SocketAddr,
) -> Result<AnnounceRequest, AnnounceError> {
    let missing_params: Vec<String> = get_missing_mandatory_params(&params);
    if !missing_params.is_empty() {
        return Err(AnnounceError::MissingParam(missing_params.join(", ")));
    }

    let info_hash: Vec<u8> = params.get("info_hash").unwrap().clone().into_bytes();
    let peer_id: Vec<u8> = params.get("peer_id").unwrap().clone().into_bytes();
    let uploaded: u32 = parse_entry_to_u32(&params, "uploaded")?;
    let downloaded: u32 = parse_entry_to_u32(&params, "downloaded")?;
    let left: u32 = parse_entry_to_u32(&params, "left")?;
    let listening_port: u32 = parse_entry_to_u32(&params, "port")?;

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
    let mandatory_params: Vec<String> =
        vec!["info_hash", "peer_id", "uploaded", "downloaded", "left"]
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

pub fn is_active_peer(last_announce: DateTime<Local>, interval: u32) -> bool {
    let time_between_announces: chrono::Duration = Local::now() - last_announce;
    time_between_announces < chrono::Duration::seconds((2 * interval).into())
}

pub fn has_completed(request: &AnnounceRequest) -> bool {
    request.event == TrackerEvent::Completed
}

pub fn is_peer_stopping(request: &AnnounceRequest) -> bool {
    request.event == TrackerEvent::Stopped
}

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
