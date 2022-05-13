use super::types::*;
use std::fmt::Display;

#[derive(Debug)]
/// The error type that is returned by the decoder
pub enum BencodeDecoderError {
    /// an digit from 0 to 9 was expected but got another byte instead
    DecodeInt(u8),
    /// a string key was expected but got another [`BencodeDecodedValue`] instead as a key instead
    UnexpectedDictionaryKey(BencodeDecodedValue),
    /// slice of decoded bytes ended before a valid ending token was found
    UnexpectedEndOfStream,
}

impl Display for BencodeDecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BencodeDecoderError::DecodeInt(byte) => {
                write!(f, "Bencoder: Error decoding byte {} as integer", byte)
            }
            BencodeDecoderError::UnexpectedEndOfStream => {
                write!(f, "Bencoder: Unexpected end of stream")
            }
            BencodeDecoderError::UnexpectedDictionaryKey(wrong_key) => {
                write!(f, "Bencoder: Unexpected dictionary key {:?}", wrong_key)
            }
        }
    }
}
