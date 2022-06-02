use super::constants::*;
use super::errors::HttpsConnectionError;
use super::types::HttpService;
use crate::boxed_result::BoxedResult;
use log::*;
use native_tls::TlsConnector;
use native_tls::TlsStream;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub struct HttpsConnection {
    stream: TlsStream<TcpStream>,
    host: String,
    max_retries: u8,
}

impl HttpsConnection {
    pub fn from_url(url: &str) -> Result<HttpsConnection, HttpsConnectionError> {
        debug!("Creating https connection from url: {}", url);
        let host = HttpsConnection::url_to_host(url)?;
        let connector = TlsConnector::new()?;
        let stream = TcpStream::connect(format!("{}:{}", host, HTTPS_PORT))?;
        stream.set_write_timeout(Some(Duration::new(REQUEST_TIMEOUT, 0)))?;
        stream.set_read_timeout(Some(Duration::new(REQUEST_TIMEOUT, 0)))?;
        let stream = connector.connect(&host, stream)?;
        Ok(HttpsConnection {
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
            .ok_or_else(|| HttpsConnectionError(format!("Missign URN in URL: {}", url)))?;
        let host = urn.split(HOST_SEPARATOR).next().ok_or_else(|| {
            HttpsConnectionError(format!("Could not separate HOST from URN: {}", url))
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
            Err(Box::new(HttpsConnectionError(format!(
                "Could not find response body in response: {}",
                String::from_utf8_lossy(&response)
            ))))
        }
    }
}

impl HttpService for HttpsConnection {
    fn get(&mut self, path: &str, query_params: &str) -> Result<Vec<u8>, HttpsConnectionError> {
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
                        return Err(HttpsConnectionError(format!(
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
pub struct MockHttpsConnection {
    pub read_bytes: Vec<u8>,
}

#[cfg(test)]
impl HttpService for MockHttpsConnection {
    fn get(&mut self, _path: &str, _query_params: &str) -> Result<Vec<u8>, HttpsConnectionError> {
        Ok(self.read_bytes.clone())
    }
}
