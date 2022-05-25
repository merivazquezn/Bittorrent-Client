use std::error;
use std::fmt;

#[derive(Debug)]
pub struct HttpsConnectionError(pub String);

impl error::Error for HttpsConnectionError {}

// impl From Box<dyn Error> trait for HttpsConnectionError
impl From<Box<dyn error::Error>> for HttpsConnectionError {
    fn from(error: Box<dyn error::Error>) -> Self {
        HttpsConnectionError(format!("{}", error))
    }
}

impl From<native_tls::Error> for HttpsConnectionError {
    fn from(error: native_tls::Error) -> Self {
        HttpsConnectionError(format!("Native tls connection creation error: {}", error))
    }
}

impl From<std::io::Error> for HttpsConnectionError {
    fn from(error: std::io::Error) -> Self {
        HttpsConnectionError(format!("Connection creation error: {}", error))
    }
}

impl From<native_tls::HandshakeError<std::net::TcpStream>> for HttpsConnectionError {
    fn from(error: native_tls::HandshakeError<std::net::TcpStream>) -> Self {
        HttpsConnectionError(format!("Connection handshake error: {}", error))
    }
}

impl fmt::Display for HttpsConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
