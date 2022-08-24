use super::constants::HTTP_CONTENT_LENGTH;
use super::constants::HTTP_CONTENT_TYPE;
use super::constants::HTTP_HEADER_SEPARATOR;
use super::constants::HTTP_OK_RESPONSE;
use super::constants::KEY_VALUE_SEPARATOR;
use super::constants::QUERY_PARAMS_SEPARATOR;
use super::constants::QUERY_PARAMS_START;
use super::errors::HttpError;
use std::collections::HashMap;

pub fn parse_query_params_from_path(path: &str) -> Result<HashMap<String, String>, HttpError> {
    if !request_has_query_params(path) {
        return Ok(HashMap::new());
    }

    let query_params = path.split(QUERY_PARAMS_START).nth(1);
    match query_params {
        Some(query_params) => {
            let mut params: HashMap<String, String> = HashMap::new();
            for param in query_params.split(QUERY_PARAMS_SEPARATOR) {
                let key_value: Vec<&str> = param.split(KEY_VALUE_SEPARATOR).collect();
                if key_value.len() != 2 {
                    return Err(HttpError::InvalidRequest(format!(
                        "Invalid query param: {}",
                        param
                    )));
                }
                if key_value[0] == "peer_id" || key_value[0] == "info_hash" {
                    params.insert(
                        key_value[0].to_string(),
                        to_hex(&from_urlencoded(key_value[1])?),
                    );
                } else {
                    params.insert(key_value[0].to_string(), key_value[1].to_string());
                }
            }
            Ok(params)
        }
        None => Err(HttpError::InvalidRequest(format!(
            "Invalid query params: {}",
            path
        ))),
    }
}

fn request_has_query_params(path: &str) -> bool {
    path.contains('?')
}

pub fn get_path_from_request(request: &str) -> Result<String, HttpError> {
    match request.trim_start_matches("GET /").split(' ').next() {
        Some(path) => Ok(path.to_string()),
        None => Err(HttpError::InvalidRequest("Invalid path".to_string())),
    }
}

pub fn endpoint_from_path(path: &str) -> Result<String, HttpError> {
    match path.split(QUERY_PARAMS_START).next() {
        Some(endpoint) => Ok(endpoint.to_string()),
        None => Err(HttpError::InvalidRequest("Invalid path".to_string())),
    }
}

pub fn is_get_request(request: &[u8]) -> bool {
    request.starts_with(b"GET")
}

pub fn request_as_str(request: &[u8]) -> Result<&str, HttpError> {
    Ok(std::str::from_utf8(request)?)
}

pub fn format_http_response(content: Vec<u8>, content_type: String) -> String {
    format!(
        "{}{}{}{}{}{}{}{}{}",
        HTTP_OK_RESPONSE,
        HTTP_HEADER_SEPARATOR,
        HTTP_CONTENT_LENGTH,
        content.len(),
        HTTP_HEADER_SEPARATOR,
        HTTP_CONTENT_TYPE,
        content_type,
        HTTP_HEADER_SEPARATOR,
        HTTP_HEADER_SEPARATOR
    )
}

// Transforms a url-encoded String to a vector of bytes
fn from_urlencoded(urlencoded: &str) -> Result<Vec<u8>,HttpError> {
    let mut bytes = vec![];
    let mut index = 0;
    while index < urlencoded.len() {
        if urlencoded.get(index..index + 1) == Some("%") {
            let hex = urlencoded.get(index + 1..index + 3).unwrap();
            let hex_byte = u8::from_str_radix(hex, 16)?;
            bytes.push(hex_byte);
            index += 3;
        } else {
            bytes.push(urlencoded.as_bytes()[index]);
            index += 1;
        }
    }
    Ok(bytes)
}

// transform a vector of bytes into a string of hexadecimal characters
fn to_hex(bytes: &[u8]) -> String {
    let mut hex = String::new();
    for byte in bytes {
        hex.push_str(&format!("{:02x}", byte));
    }
    hex
}
