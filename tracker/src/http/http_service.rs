use super::constants::HTTP_NOT_FOUND_RESPONSE;
use super::utils::endpoint_from_path;
use super::utils::format_http_response;
use super::utils::get_path_from_request;
use super::utils::is_get_request;
use super::utils::parse_query_params_from_path;
use super::utils::request_as_str;
use super::HttpError;
use bittorrent_rustico::logger::CustomLogger;
use std::io::{Read, Write};
use std::{collections::HashMap, net::TcpStream};

const LOGGER: CustomLogger = CustomLogger::init("HTTP Service");
#[derive(Debug)]
pub struct HttpGetRequest {
    pub params: HashMap<String, String>,
    pub path: String,
}

pub trait IHttpService: Send {
    fn parse_request(&mut self) -> Result<HttpGetRequest, HttpError>;

    fn send_ok_response(&mut self, content: Vec<u8>, content_type: String)
        -> Result<(), HttpError>;

    fn send_not_found(&mut self) -> Result<(), HttpError>;
}
pub struct HttpService {
    stream: TcpStream,
}

impl HttpService {
    pub fn from_stream(stream: TcpStream) -> HttpService {
        HttpService { stream }
    }

    fn send_response(&mut self, response: Vec<u8>) -> Result<(), HttpError> {
        self.stream.write_all(&response)?;
        self.stream.flush()?;
        Ok(())
    }
}

impl IHttpService for HttpService {
    fn parse_request(&mut self) -> Result<HttpGetRequest, HttpError> {
        let mut read_buffer: [u8; 2048] = [0; 2048];
        LOGGER.info_str("Parsing request...");
        let _ = self.stream.read(&mut read_buffer)?;
        let buffer: Vec<u8> = read_buffer.to_vec();
        LOGGER.info_str("Finished reading request");
        if !is_get_request(&buffer) {
            return Err(HttpError::InvalidRequest(
                request_as_str(&buffer).to_string(),
            ));
        }

        let request: &str = request_as_str(&buffer);
        let path: String = get_path_from_request(request)?;
        let params: HashMap<String, String> = parse_query_params_from_path(&path)?;
        let endpoint: String = endpoint_from_path(&path)?;

        Ok(HttpGetRequest {
            params,
            path: endpoint,
        })
    }

    fn send_not_found(&mut self) -> Result<(), HttpError> {
        let response: Vec<u8> = HTTP_NOT_FOUND_RESPONSE.to_string().into_bytes();
        self.send_response(response)
    }

    fn send_ok_response(
        &mut self,
        mut content: Vec<u8>,
        content_type: String,
    ) -> Result<(), HttpError> {
        let response: String = format_http_response(content.clone(), content_type);
        let mut response = response.as_bytes().to_vec();
        response.append(&mut content);
        self.send_response(response)
    }
}
