pub use super::super::bencode::*;
use std::fmt::Display;
#[derive(Debug)]
///The error type that is returned if theres a problem parsing the Metainfo
pub enum MetainfoParserError {
    IoError(std::io::Error),
    ///There was an error while decoding the bytes received
    DecodeError(BencodeDecoderError),
    ///A necessary key was not found in the Bencode-Decoded Dictionary
    MetainfoKeyNotFound(String),
    ///An unexpected value type was found while building the Metainfo struct
    UnexpectedBencodeDecodedValue(BencodeDecoderError),
    //There was a problem parsing a byte array into a String from UTF-8
    UTF8Error,
}

impl From<BencodeDecoderError> for MetainfoParserError {
    fn from(error: BencodeDecoderError) -> Self {
        match error {
            BencodeDecoderError::WrongExpectedValue(_, _) => {
                MetainfoParserError::UnexpectedBencodeDecodedValue(error)
            }
            _ => MetainfoParserError::DecodeError(error),
        }
    }
}

impl From<std::io::Error> for MetainfoParserError {
    fn from(error: std::io::Error) -> Self {
        MetainfoParserError::IoError(error)
    }
}

impl Display for MetainfoParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetainfoParserError::DecodeError(error) => {
                writeln!(
                    f,
                    "BencodeDecoder encountered an error while decoding: {}",
                    error
                )
            }
            MetainfoParserError::MetainfoKeyNotFound(key) => {
                writeln!(
                    f,
                    "Necessary key '{}' was not in Bencode-Decoded Dictionary",
                    key
                )
            }
            MetainfoParserError::UnexpectedBencodeDecodedValue(error) => {
                writeln!(
                    f,
                    "An unexpected value type was found while building Metainfo: '{}'",
                    error
                )
            }
            MetainfoParserError::UTF8Error => {
                writeln!(
                    f,
                    "Found that byte array was not UTF-8 encoded when parsing to String ",
                )
            }
            MetainfoParserError::IoError(error) => {
                writeln!(f, "IO error while reading the file: {}", error)
            }
        }
    }
}
