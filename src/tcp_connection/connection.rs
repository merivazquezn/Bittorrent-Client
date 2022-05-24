pub trait TcpConnection {
    fn write(&mut self, data: &[u8]) -> Result<(), TcpConnectionError>;

    fn read(&mut self, buf: &mut Vec<u8>) -> Result<usize, TcpConnectionError>;
}

pub enum TcpConnectionError {
    WriteError,
    ReadError,
}

//display
impl std::fmt::Display for TcpConnectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TcpConnectionError::WriteError => write!(f, "Write Error"),
            TcpConnectionError::ReadError => write!(f, "Read Error"),
        }
    }
}
