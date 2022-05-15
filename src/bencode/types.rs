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
/// let decoded_value = BencodeDecodedValue::String(String::from("hola"));
///
/// assert_eq!(decoded_value.get_as_string().unwrap(), "hola");
/// ```
pub enum BencodeDecodedValue {
    String(String),
    Integer(i64),
    List(Vec<BencodeDecodedValue>),
    Dictionary(HashMap<String, BencodeDecodedValue>),
    End,
}

impl BencodeDecodedValue {
    // implement all the methods of the matter get_as_string, get_as_integer, get_as_list, get_as_dictionary
    // and get_as_end

    pub fn get_as_string(&self) -> Result<&String, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::String(value) => Ok(value),
            _ => Err(BencodeDecoderError::WrongExpectedValue(
                self.clone(),
                String::from("BencodeDecodedValue::String"),
            )),
        }
    }
    pub fn get_as_integer(&self) -> Result<&i64, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::Integer(value) => Ok(value),
            _ => Err(BencodeDecoderError::WrongExpectedValue(
                self.clone(),
                String::from("BencodeDecodedValue::Integer"),
            )),
        }
    }
    pub fn get_as_list(&self) -> Result<&Vec<BencodeDecodedValue>, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::List(value) => Ok(value),
            _ => Err(BencodeDecoderError::WrongExpectedValue(
                self.clone(),
                String::from("BencodeDecodedValue::List"),
            )),
        }
    }
    pub fn get_as_dictionary(
        &self,
    ) -> Result<&HashMap<String, BencodeDecodedValue>, BencodeDecoderError> {
        match self {
            BencodeDecodedValue::Dictionary(value) => Ok(value),
            _ => Err(BencodeDecoderError::WrongExpectedValue(
                self.clone(),
                String::from("BencodeDecodedValue::Dictionary"),
            )),
        }
    }
}
