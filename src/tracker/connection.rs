extern crate native_tls;
use super::types::RequestParameters;
use crate::bencode::*;
use native_tls::TlsConnector;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;

const SEPARATOR: &[u8] = b"\r\n\r\n";

// from u8 into urlencoded string. any byte not in the set 0-9, a-z, A-Z, '.', '-', '_' and '~', must be encoded using the "%nn" format, where nn is the hexadecimal value of the byte.
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

fn parameters_to_querystring(parameters: RequestParameters) -> String {
    let parameters = params_to_dic(parameters);
    let mut querystring = String::new();
    for (key, value) in parameters {
        querystring.push_str(&format!("{}={}&", key, value));
    }
    querystring.pop();
    querystring
}

fn bencode_response(bytes: &[u8]) -> Vec<u8> {
    let start_index = bytes.windows(4).position(|arr| arr == SEPARATOR);
    start_index.map(|i| bytes[i + 4..].to_vec()).unwrap()
}

pub fn get_peer_list(
    parameters: RequestParameters,
) -> Result<BencodeDecodedValue, BencodeDecoderError> {
    let connector = TlsConnector::new().unwrap();
    let stream = TcpStream::connect("torrent.ubuntu.com:443").unwrap();
    let mut stream = connector.connect("torrent.ubuntu.com", stream).unwrap();

    let mut request = String::new();

    request.push_str(&format!(
        "GET {}?{} HTTP/1.0\r\n",
        "/announce",
        parameters_to_querystring(parameters)
    ));
    request.push_str("Host: torrent.ubuntu.com");
    request.push_str("\r\n");
    request.push_str("\r\n");

    stream.write_all(request.as_bytes()).unwrap();
    let mut res: Vec<u8> = Vec::new();
    stream.read_to_end(&mut res).unwrap();

    let bytes_after_rn = bencode_response(res.as_slice());
    let decoded = decode(bytes_after_rn.as_slice())?;
    Ok(decoded)
}
