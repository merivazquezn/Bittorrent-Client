use std::error::Error;

pub trait TcpConnection {
    fn write(&mut self, data: &[u8]) -> Result<(), Box<dyn Error + Send + Sync>>;

    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, Box<dyn Error + Send + Sync>>;
}
