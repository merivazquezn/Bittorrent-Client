use super::constants::*;
use super::types::RequestParameters;
use std::collections::HashMap;
use std::str;

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

// Builds the querystring to use in the tracker request form the RequestParameters struct
pub fn parameters_to_querystring(parameters: &RequestParameters) -> String {
    let parameters = params_to_dic(parameters);
    let mut querystring = String::new();
    for (key, value) in parameters {
        querystring.push_str(&format!("{}={}&", key, value));
    }
    querystring.pop();
    querystring
}

// Gets the actual data from the tracker response, leaving out the HTTP headers
pub fn bencode_response(bytes: &[u8]) -> Vec<u8> {
    let start_index = bytes.windows(4).position(|arr| arr == SEPARATOR);
    start_index.map(|i| bytes[i + 4..].to_vec()).unwrap()
}

// transforms a slice of bytes into its utf-8 representation
pub fn u8_to_string(bytes: &[u8]) -> String {
    str::from_utf8(bytes).unwrap().to_string()
}
