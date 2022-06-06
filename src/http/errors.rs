use std::error;
use std::fmt;

#[derive(Debug)]
pub struct HttpsServiceError(pub String);

impl error::Error for HttpsServiceError {}

// impl From Box<dyn Error> trait for HttpsServiceError
impl From<Box<dyn error::Error>> for HttpsServiceError {
    fn from(error: Box<dyn error::Error>) -> Self {
        HttpsServiceError(format!("{}", error))
    }
}

impl From<native_tls::Error> for HttpsServiceError {
    fn from(error: native_tls::Error) -> Self {
        HttpsServiceError(format!("Native tls connection creation error: {}", error))
    }
}

impl From<std::io::Error> for HttpsServiceError {
    fn from(error: std::io::Error) -> Self {
        HttpsServiceError(format!("Connection creation error: {}", error))
    }
}

impl From<native_tls::HandshakeError<std::net::TcpStream>> for HttpsServiceError {
    fn from(error: native_tls::HandshakeError<std::net::TcpStream>) -> Self {
        HttpsServiceError(format!("Connection handshake error: {}", error))
    }
}

impl fmt::Display for HttpsServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
