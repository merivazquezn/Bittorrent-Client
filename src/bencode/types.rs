use super::errors::*;
use std::collections::HashMap;
#[derive(Debug, Clone, PartialEq, Eq)]
/// The type that is returned by the decoder
/// and is used to represent the decoded bencode value
/// It can be either a string, integer, list or dictionary
///
/// ## Example
/// ```
/// use std::collections::HashMap;
/// use bittorrent_rustico::bencode::BencodeDecodedValue;
/// let decoded_value = BencodeDecodedValue::String(b"hola".to_vec());
///
/// assert_eq!(decoded_value.get_as_string().unwrap(), b"hola");
/// ```
pub enum BencodeDecodedValue {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<BencodeDecodedValue>),
    Dictionary(HashMap<Vec<u8>, BencodeDecodedValue>),
    End,
}

impl BencodeDecodedValue {
    pub fn get_as_string(&self) -> Result<&Vec<u8>, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::String(value) => Ok(value),
            _ => Err(BencodeDecoderError(format!(
                "Expected a string, but got {:?}",
                self
            ))),
        }
    }
    pub fn get_as_integer(&self) -> Result<&i64, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::Integer(value) => Ok(value),
            _ => Err(BencodeDecoderError(format!(
                "Expected an integer, but got {:?}",
                self
            ))),
        }
    }
    pub fn get_as_list(&self) -> Result<&Vec<BencodeDecodedValue>, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::List(value) => Ok(value),
            _ => Err(BencodeDecoderError(format!(
                "Expected a list, but got {:?}",
                self
            ))),
        }
    }
    pub fn get_as_dictionary(
        &self,
    ) -> Result<&HashMap<Vec<u8>, BencodeDecodedValue>, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::Dictionary(value) => Ok(value),
            _ => Err(BencodeDecoderError(format!(
                "Expected a dictionary, but got {:?}",
                self
            ))),
        }
    }
}
