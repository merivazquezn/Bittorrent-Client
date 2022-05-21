use super::types::BencodeDecodedValue;
use std::fmt::Display;

#[derive(Debug)]
/// The error type that is returned by the decoder
pub enum BencodeDecoderError {
    /// an digit from 0 to 9 was expected but got another byte instead
    DecodeInt(u8),

    /// a string key was expected but got another [`BencodeDecodedValue`] instead as a key instead
    UnexpectedDictionaryKey(BencodeDecodedValue),
    /// tried to extract the wrong value of bencodeDecodedValue
    WrongExpectedValue(BencodeDecodedValue, String),
    /// an invalid format for a number was given, for example: 054
    InvalidInt,
    /// slice of decoded bytes ended before a valid ending token was found
    UnexpectedEndOfStream,
}

impl Display for BencodeDecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BencodeDecoderError::DecodeInt(byte) => {
                write!(f, "Error decoding byte {} as integer", byte)
            }
            BencodeDecoderError::UnexpectedEndOfStream => {
                write!(f, "Unexpected end of stream")
            }
            BencodeDecoderError::UnexpectedDictionaryKey(wrong_key) => {
                write!(f, "Unexpected dictionary key {:?}", wrong_key)
            }
            BencodeDecoderError::WrongExpectedValue(actual_value, expected_value) => {
                write!(
                    f,
                    "Expected value was {:?}, but real value is {:?}",
                    expected_value, actual_value
                )
            }
            BencodeDecoderError::InvalidInt => write!(f, "Invalid integer format"),
        }
    }
}
