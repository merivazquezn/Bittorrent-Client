use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
/// The type that is returned by the decoder
/// and is used to represent the decoded bencode value
/// It can be either a string, integer, list or dictionary
///
/// ## Example
/// ```
/// use bittorrent_rustico::bencode::BencodeDecodedValue;
/// let decoded_value = BencodeDecodedValue::String(b"hola".to_vec());
///
/// if let BencodeDecodedValue::String(value) = decoded_value {
///     let string = String::from_utf8(value).unwrap();
///     assert_eq!(String::from("hola"), string);
/// }
/// ```
pub enum BencodeDecodedValue {
    String(Vec<u8>),
    Integer(i64),
    List(Vec<BencodeDecodedValue>),
    Dictionary(HashMap<Vec<u8>, BencodeDecodedValue>),
    End,
}
