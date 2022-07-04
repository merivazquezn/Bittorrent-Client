use super::types::RequestParameters;
use crate::application_errors::ApplicationError;
use crate::client::ClientInfo;
use crate::http::HttpsService;
use crate::logger::CustomLogger;
use crate::tracker::ITrackerService;
use crate::tracker::TrackerResponse;
use crate::tracker::TrackerService;
use crate::ui::UIMessageSender;
use std::collections::HashMap;

const LOGGER: CustomLogger = CustomLogger::init("Tracker");
const WANTED_CONNECTIONS: u32 = 100;

pub fn get_response_from_tracker(
    client_info: &mut ClientInfo,
    ui_message_sender: UIMessageSender,
) -> Result<(TrackerResponse, TrackerService), ApplicationError> {
    LOGGER.info(format!(
        "Fetching Peers from tracker at: {}",
        client_info.metainfo.announce
    ));
    let http_service = HttpsService::from_url(&client_info.metainfo.announce)?;
    let mut tracker_service = TrackerService::from_metainfo(
        &client_info.metainfo,
        client_info.config.listen_port,
        &client_info.peer_id,
        Box::new(http_service),
    );
    let tracker_response = tracker_service.get_response()?;
    ui_message_sender.send_initial_peers(tracker_response.peers.len() as u32);
    LOGGER.info(format!(
        "Received {} peers from tracker",
        tracker_response.peers.len()
    ));
    match tracker_response.interval {
        Some(interval) => {
            LOGGER.info(format!("Tracker interval: {:?}", interval));
        }
        None => {
            LOGGER.info_str("Tracker interval not set");
        }
    }

    Ok((tracker_response, tracker_service))
}

// Transforms a slice of bytes into an url-encoded String
fn to_urlencoded(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| {
            if b.is_ascii_alphanumeric() || *b == b'.' || *b == b'-' || *b == b'_' || *b == b'~' {
                String::from(*b as char)
            } else {
                format!("%{:02x}", *b)
            }
        })
        .collect()
}

// Maps RequestParameters to a Hashmap where all the values of the type are represented as strings
fn params_to_dic(params: &RequestParameters) -> HashMap<String, String> {
    let mut dictionary = HashMap::new();
    dictionary.insert("info_hash".to_string(), to_urlencoded(&params.info_hash));
    dictionary.insert("peer_id".to_string(), to_urlencoded(&params.peer_id));
    dictionary.insert("port".to_string(), params.port.to_string());
    dictionary.insert("uploaded".to_string(), params.uploaded.to_string());
    dictionary.insert("downloaded".to_string(), params.downloaded.to_string());
    dictionary.insert("left".to_string(), params.left.to_string());
    dictionary.insert("event".to_string(), String::from("started"));
    dictionary
}

/// Builds the querystring to use in the tracker request form the RequestParameters struct
pub fn parameters_to_querystring(parameters: &RequestParameters) -> String {
    let parameters = params_to_dic(parameters);
    let mut querystring = String::new();
    for (key, value) in parameters {
        querystring.push_str(&format!("{}={}&", key, value));
    }
    querystring.push_str(&format!("{}={}&", "compact", "1"));
    querystring.push_str(&format!("{}={}", "numwant", WANTED_CONNECTIONS));

    querystring
}

/// transforms a slice of bytes into its utf-8 representation
pub fn u8_to_string(bytes: &[u8]) -> Option<String> {
    String::from_utf8(bytes.into()).ok()
}
