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
/// if let BencodeDecodedValue::String(value) = decoded_value {
///     assert_eq!(String::from("hola"), value);
/// }
///
/// let decoded_dict = BencodeDecodedValue::Dictionary(
///    HashMap::from([
///       (String::from("key1"), BencodeDecodedValue::String(String::from("value1"))),
///      (String::from("key2"), BencodeDecodedValue::String(String::from("value2"))),
///   ])
/// );
///
/// let second_element = decoded_dict.get("key2").unwrap();
///
/// if let BencodeDecodedValue::String(value) = second_element {
///    assert_eq!("value2", value);
/// } else {
///     panic!("Expected BencodeDecodedValue::String but got {:?}", second_element);
/// }
///
/// ```
pub enum BencodeDecodedValue {
    String(String),
    Integer(i64),
    List(Vec<BencodeDecodedValue>),
    Dictionary(HashMap<String, BencodeDecodedValue>),
    End,
}
impl BencodeDecodedValue {
    // if bencodeDecodedValue is a dictionary, return the value associated with the key
    pub fn get(&self, key: &str) -> Option<&BencodeDecodedValue> {
        match self {
            BencodeDecodedValue::Dictionary(dictionary) => dictionary.get(key),
            _ => None,
        }
    }
}
