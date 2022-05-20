extern crate native_tls;
use super::errors::*;
use super::types::Peer;
use super::types::RequestParameters;
use super::types::TrackerResponse;
use crate::bencode::BencodeDecodedValue;
use crate::bencode::*;
use log::*;
use native_tls::TlsConnector;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::str;

const SEPARATOR: &[u8] = b"\r\n\r\n";
const PEERS: &[u8] = b"peers";
const IP: &[u8] = b"ip";
const PORT: &[u8] = b"port";
const PEER_ID: &[u8] = b"peer id";

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
fn params_to_dic(params: RequestParameters) -> HashMap<String, String> {
    let mut dictionary = HashMap::new();
    dictionary.insert(
        "info_hash".to_string(),
        to_urlencoded(params.info_hash.as_slice()),
    );
    dictionary.insert(
        "peer_id".to_string(),
        to_urlencoded(params.peer_id.as_slice()),
    );
    dictionary.insert("port".to_string(), params.port.to_string());
    dictionary.insert("uploaded".to_string(), params.uploaded.to_string());
    dictionary.insert("downloaded".to_string(), params.downloaded.to_string());
    dictionary.insert("left".to_string(), params.left.to_string());
    dictionary.insert("event".to_string(), String::from("started"));
    dictionary
}

// Builds the querystring to use in the tracker request form the RequestParameters struct
fn parameters_to_querystring(parameters: RequestParameters) -> String {
    let parameters = params_to_dic(parameters);
    let mut querystring = String::new();
    for (key, value) in parameters {
        querystring.push_str(&format!("{}={}&", key, value));
    }
    querystring.pop();
    querystring
}

// Gets the actual data from the tracker response, leaving out the HTTP headers
fn bencode_response(bytes: &[u8]) -> Vec<u8> {
    let start_index = bytes.windows(4).position(|arr| arr == SEPARATOR);
    start_index.map(|i| bytes[i + 4..].to_vec()).unwrap()
}

// transforms a slice of bytes into its utf-8 representation
fn u8_to_string(bytes: &[u8]) -> String {
    str::from_utf8(bytes).unwrap().to_string()
}

// Builds the TrackerResponse from the bencoded data
fn parse_response(bencoded_response: BencodeDecodedValue) -> Result<TrackerResponse, TrackerError> {
    // Parse decoded response
    let response_dic = bencoded_response.get_as_dictionary()?;
    let benencoded_peers_list = match response_dic.get(PEERS) {
        Some(peers) => peers.get_as_list()?,
        None => return Err(TrackerError::InvalidResponse),
    };

    let mut peer_list: Vec<Peer> = Vec::new();

    for value in benencoded_peers_list.iter() {
        let peer_dic = value.get_as_dictionary()?;
        let peer_ip = match peer_dic.get(IP) {
            Some(ip) => ip.get_as_string()?,
            None => return Err(TrackerError::InvalidResponse),
        };
        let peer_port = match peer_dic.get(PORT) {
            Some(port) => port.get_as_integer()?,
            None => return Err(TrackerError::InvalidResponse),
        };
        let peer_id = match peer_dic.get(PEER_ID) {
            Some(peer_id) => peer_id.get_as_string()?,
            None => return Err(TrackerError::InvalidResponse),
        };
        let peer = Peer {
            ip: u8_to_string(peer_ip),
            port: *peer_port,
            peer_id: peer_id.to_vec(),
        };

        peer_list.push(peer);
    }

    Ok(TrackerResponse { peers: peer_list })
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
/// ```
/// use bittorrent_rustico::tracker::{get_peer_list, Event, RequestParameters};
/// use hex::FromHex;
///
/// let info_hash = <[u8; 20]>::from_hex("2c6b6858d61da9543d4231a71db4b1c9264b0685").unwrap();
/// let peer_id = info_hash;
/// let params = RequestParameters {
///        info_hash: info_hash.to_vec(),
///        peer_id: peer_id.to_vec(),
///        port: 6881,
///        uploaded: 0,
///        downloaded: 0,
///        left: 0,
///        event: Event::Started,
/// };
///
/// let response = get_peer_list(params).unwrap();
///
/// println!("{:?}", response);
/// ```
///
pub fn get_peer_list(parameters: RequestParameters) -> Result<TrackerResponse, TrackerError> {
    info!("Requesting peer list from tracker");
    let connector = TlsConnector::new()?;
    let stream = TcpStream::connect("torrent.ubuntu.com:443")?;
    let mut stream = connector.connect("torrent.ubuntu.com", stream)?;

    let mut request = String::new();

    request.push_str(&format!(
        "GET {}?{} HTTP/1.0\r\n",
        "/announce",
        parameters_to_querystring(parameters)
    ));
    request.push_str("Host: torrent.ubuntu.com");
    request.push_str("\r\n\r\n");

    stream.write_all(request.as_bytes())?;
    let mut res: Vec<u8> = Vec::new();
    stream.read_to_end(&mut res)?;

    let bytes_after_rn = bencode_response(res.as_slice());
    let decoded: BencodeDecodedValue = decode(bytes_after_rn.as_slice())?;

    match parse_response(decoded) {
        Ok(response) => Ok(response),
        Err(error) => Err(error),
    }
}
