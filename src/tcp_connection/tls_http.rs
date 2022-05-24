use super::constants::*;
use super::*;
use native_tls::TlsConnector;
use native_tls::TlsStream;
use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;

pub struct TlsHttpConnection {
    stream: TlsStream<TcpStream>,
}

impl TlsHttpConnection {
    pub fn create(host: &str) -> Result<TlsHttpConnection, Box<dyn Error + Send + Sync>> {
        let connector = TlsConnector::new()?;
        let stream = TcpStream::connect(format!("{}:{}", host, HTTPS_PORT))?;
        let stream = connector.connect(host, stream)?;
        Ok(TlsHttpConnection { stream })
    }
}

impl TcpConnection for TlsHttpConnection {
    fn write(&mut self, data: &[u8]) -> Result<(), TcpConnectionError> {
        self.stream
            .write_all(data)
            .map_err(|err| TcpConnectionError::WriteError(err.to_string()))
    }

    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, TcpConnectionError> {
        self.stream
            .read_to_end(buf)
            .map_err(|err| TcpConnectionError::ReadError(err.to_string()))
    }
}
