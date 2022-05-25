use std::error;
use std::fmt;

#[derive(Debug)]
pub struct BencodeDecoderError(pub String);

impl error::Error for BencodeDecoderError {}

impl From<Box<dyn error::Error>> for BencodeDecoderError {
    fn from(error: Box<dyn error::Error>) -> Self {
        BencodeDecoderError(format!("{}", error))
    }
}

impl fmt::Display for BencodeDecoderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Bencode Decoder Error: {}", self.0)
    }
}
