use std::collections::HashMap;
use std::fmt::Display;
use std::io;
use std::num;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum BencodeDecodedValue {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<BencodeDecodedValue>),
    Dictionary(HashMap<Vec<u8>, BencodeDecodedValue>),
}

#[derive(Debug)]
pub enum BencodeDecoderError {
    IoError(io::Error),
    ParseError(num::ParseIntError),
}

impl Display for BencodeDecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BencodeDecoderError::IoError(io_error) => write!(f, "Bencoder: {}", io_error),
            BencodeDecoderError::ParseError(parse_int_error) => {
                write!(f, "Bencoder: {}", parse_int_error)
            }
        }
    }
}

impl From<io::Error> for BencodeDecoderError {
    fn from(error: io::Error) -> Self {
        BencodeDecoderError::IoError(error)
    }
}

impl From<num::ParseIntError> for BencodeDecoderError {
    fn from(error: num::ParseIntError) -> Self {
        BencodeDecoderError::ParseError(error)
    }
}
