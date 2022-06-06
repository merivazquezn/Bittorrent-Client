use super::constants::*;
use super::errors::HttpsServiceError;
use super::types::IHttpService;
use crate::boxed_result::BoxedResult;
use log::*;
use native_tls::TlsConnector;
use native_tls::TlsStream;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct HttpsService {
    stream: TlsStream<TcpStream>,
    host: String,
    max_retries: u8,
}

impl HttpsService {
    pub fn from_url(url: &str) -> Result<HttpsService, HttpsServiceError> {
        debug!("Creating https connection from url: {}", url);
        let host = HttpsService::url_to_host(url)?;
        let connector = TlsConnector::new()?;
        let stream = TcpStream::connect(format!("{}:{}", host, HTTPS_PORT))?;
        stream.set_write_timeout(Some(Duration::new(REQUEST_TIMEOUT, 0)))?;
        stream.set_read_timeout(Some(Duration::new(REQUEST_TIMEOUT, 0)))?;
        let stream = connector.connect(&host, stream)?;
        Ok(HttpsService {
            stream,
            host,
            max_retries: MAX_RETRIES,
        })
    }

    pub fn response_body(&self, bytes: &[u8]) -> Option<Vec<u8>> {
        let start_index = bytes.windows(4).position(|arr| arr == SEPARATOR);
        start_index.map(|i| bytes[i + 4..].to_vec())
    }

    fn url_to_host(url: &str) -> BoxedResult<String> {
        let urn = url
            .split(URN_SEPARATOR)
            .nth(1)
            .ok_or_else(|| HttpsServiceError(format!("Missign URN in URL: {}", url)))?;
        let host = urn.split(HOST_SEPARATOR).next().ok_or_else(|| {
            HttpsServiceError(format!("Could not separate HOST from URN: {}", url))
        })?;
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
