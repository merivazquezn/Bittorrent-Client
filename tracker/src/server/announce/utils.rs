use super::{AnnounceRequest, TrackerEvent};
use crate::server::errors::AnnounceError;
use std::collections::HashMap;

const DEFAULT_NUMWANT: u32 = 50;

pub fn parse_request_from_params(
    params: HashMap<String, String>,
) -> Result<AnnounceRequest, AnnounceError> {
    let missing_params: Vec<String> = get_missing_mandatory_params(&params);
    if !missing_params.is_empty() {
        return Err(AnnounceError::MissingParam(missing_params.join(", ")));
    }

    let info_hash: Vec<u8> = params.get("info_hash").unwrap().clone().into_bytes();
    let peer_id: Vec<u8> = params.get("peer_id").unwrap().clone().into_bytes();
    let port: u16 = parse_entry_to_u32(&params, "port")? as u16;
    let uploaded: u32 = parse_entry_to_u32(&params, "uploaded")?;
    let downloaded: u32 = parse_entry_to_u32(&params, "downloaded")?;
    let left: u32 = parse_entry_to_u32(&params, "left")?;

    // ip is an optional parameter, but for the moment it will be mandatory
    let ip: String = params.get("ip").unwrap().to_string();

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
        port,
        uploaded,
        downloaded,
        left,
        ip,
        event,
        numwant,
    })
}

fn parse_entry_to_u32(params: &HashMap<String, String>, key: &str) -> Result<u32, AnnounceError> {
    Ok(params
        .get(key)
        .unwrap()
        .parse()
        .map_err(|_| AnnounceError::BadRequest))?
}

fn get_missing_mandatory_params(params: &HashMap<String, String>) -> Vec<String> {
    let mut missing_params: Vec<String> = Vec::new();
    let mandatory_params: Vec<String> = vec![
        "info_hash",
        "peer_id",
        "port",
        "uploaded",
        "downloaded",
        "left",
        "ip",
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