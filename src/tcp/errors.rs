use std::error;
use std::fmt;

#[derive(Debug)]
pub struct TcpConnectionError(pub String);

impl error::Error for TcpConnectionError {}

impl fmt::Display for TcpConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TCP connection Error: {}", self.0)
    }
}
