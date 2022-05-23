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
    fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(self.stream.write_all(data)?)
    }

    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.stream.read_to_end(buf)?)
    }
}
