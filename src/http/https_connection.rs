use super::constants::*;
use super::errors::HttpsServiceError;
use super::types::IHttpService;
use crate::boxed_result::BoxedResult;
use log::*;
use native_tls::{TlsConnector, TlsStream};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub enum CustomTcpStream {
    Https(TlsStream<TcpStream>),
    Http(TcpStream),
}

impl CustomTcpStream {
    pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize, std::io::Error> {
        match self {
            CustomTcpStream::Https(stream) => stream.read_to_end(buf),
            CustomTcpStream::Http(stream) => stream.read_to_end(buf),
        }
    }

    pub fn write_all(&mut self, buf: &[u8]) -> Result<(), std::io::Error> {
        match self {
            CustomTcpStream::Https(stream) => stream.write_all(buf),
            CustomTcpStream::Http(stream) => stream.write_all(buf),
        }
    }
}

pub struct HttpsService {
    stream: CustomTcpStream,
    host: String,
    max_retries: u8,
}

impl HttpsService {
    // if url is https, use native_tls to create the stream
    pub fn from_url(url: &str) -> Result<HttpsService, HttpsServiceError> {
        debug!("Creating https connection from url: {}", url);

        let host = HttpsService::url_to_host(url)?;
        trace!("host: {}", host);
        let stream = TcpStream::connect(&host)?;
        stream.set_write_timeout(Some(Duration::new(REQUEST_TIMEOUT, 0)))?;
        stream.set_read_timeout(Some(Duration::new(REQUEST_TIMEOUT, 0)))?;

        if url.starts_with("https://") {
            let connector = TlsConnector::new()?;
            let stream = connector.connect(&Self::remove_port_from_host(&host), stream)?;
            let stream = CustomTcpStream::Https(stream);
            Ok(HttpsService {
                stream,
                host,
                max_retries: MAX_RETRIES,
            })
        } else {
            let stream = CustomTcpStream::Http(stream);
            Ok(HttpsService {
                stream,
                host,
                max_retries: MAX_RETRIES,
            })
        }
    }

    pub fn remove_port_from_host(host: &str) -> String {
        let mut host_without_port = host.to_string();
        if let Some(index) = host_without_port.find(':') {
            host_without_port.truncate(index);
        }
        host_without_port
    }

    pub fn response_body(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        let start_index = bytes.windows(4).position(|arr| arr == SEPARATOR);
        start_index.map(|i| bytes[i + 4..].to_vec())
    }

    fn url_to_host(url: &str) -> BoxedResult<String> {
        let urn = url
            .split(URN_SEPARATOR)
            .nth(1)
            .ok_or_else(|| HttpsServiceError(format!("Missign HOST in URL: {}", url)))?;
        let host = urn
            .split(HOST_SEPARATOR)
            .next()
            .ok_or_else(|| HttpsServiceError(format!("Missing URN in URL: {}", url)))?;

        if !host.contains(':') {
            let host = host.to_string() + ":443";
            return Ok(host);
        }
        Ok(host.into())
    }

    fn try_request(&mut self, request: &str) -> BoxedResult<Vec<u8>> {
        self.stream.write_all(request.as_bytes())?;
        let mut response = vec![];
        self.stream.read_to_end(&mut response)?;
        if let Some(body) = self.response_body(&response) {
            Ok(body)
        } else {
            Err(Box::new(HttpsServiceError(format!(
                "Could not find response body in response: {}",
                String::from_utf8_lossy(&response)
            ))))
        }
    }
}

impl IHttpService for HttpsService {
    fn get(&mut self, path: &str, query_params: &str) -> Result<Vec<u8>, HttpsServiceError> {
        let request = format!(
            "GET {}?{} HTTP/1.1\r\nHost: {}\r\n\r\n",
            path, query_params, self.host
        );
        let mut retries = 0;
        loop {
            match self.try_request(&request) {
                Ok(body) => return Ok(body),
                Err(e) => {
                    if retries >= self.max_retries {
                        return Err(HttpsServiceError(format!(
                            "Could not connect to host: {}. {}",
                            self.host, e
                        )));
                    }
                    retries += 1;
                }
            }
            trace!("try number {} of tracker request", retries);
        }
    }
}

#[cfg(test)]
pub struct MockHttpsService {
    pub read_bytes: Vec<u8>,
}

#[cfg(test)]
impl IHttpService for MockHttpsService {
    fn get(&mut self, _path: &str, _query_params: &str) -> Result<Vec<u8>, HttpsServiceError> {
        Ok(self.read_bytes.clone())
    }
}
