pub use crate::bencode::BencodeDecoderError;
use std::fmt::Display;
#[derive(Debug)]
///The error type that is returned if theres a problem parsing the Metainfo
pub enum MetainfoParserError {
    IoError(std::io::Error),
    ///A necessary key was not found in the Bencode-Decoded Dictionary
    MetainfoKeyNotFound(String),
    ///An error occured when decoding the bencoded torrent
    BencodeError(String),
    //There was a problem parsing a byte array into a String from UTF-8
    UTF8Error,
    //A certain value in Info or Metainfo was invalid
    ValidationError,
}

impl From<BencodeDecoderError> for MetainfoParserError {
    fn from(error: BencodeDecoderError) -> Self {
        MetainfoParserError::BencodeError(format!("{}", error))
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
            MetainfoParserError::MetainfoKeyNotFound(key) => {
                writeln!(
                    f,
                    "Necessary key '{}' was not in Bencode-Decoded Dictionary",
                    key
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
            MetainfoParserError::BencodeError(error) => {
                writeln!(f, "Bencode error: {}", error)
            }
            MetainfoParserError::ValidationError => {
                writeln!(f, "Validation error: A Metainfo or Info value was invalid")
            }
        }
    }
}
