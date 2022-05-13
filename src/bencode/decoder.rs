use super::errors::*;
use super::types::*;
use std::collections::HashMap;
use std::slice;
const INTEGER_START_TOKEN: char = 'i';
const LIST_START_TOKEN: char = 'l';
const DICTIONARY_START_TOKEN: char = 'd';
const END_TOKEN: char = 'e';
const STRING_START_TOKEN: char = ':';
const NEGATIVE_SIGN: char = '-';

#[allow(dead_code)]
/// Decodes a bencoded byte slice into a [`BencodeDecodedValue`]
///
/// Returns the Result of the decoding, which can hold:
///
/// ## On Success
///  The decoded value as [`BencodeDecodedValue`].
///
/// ## On Error
///  A `BencodeDecoderError`.
///
/// ## Example
///
/// ```
/// use bittorrent_rustico::bencode::{decode, BencodeDecodedValue};
///
/// let decoded = decode(b"i454e").unwrap();
/// assert_eq!(decoded, BencodeDecodedValue::Integer(454));
/// ```
pub fn decode(bytes: &[u8]) -> Result<BencodeDecodedValue, BencodeDecoderError> {
    let mut bytes = bytes.iter();
    let bencoded_value = decode_and_consume_iterator(&mut bytes)?;
    Ok(bencoded_value)
}

pub fn read_integer(bytes: &mut slice::Iter<u8>) -> Result<i64, BencodeDecoderError> {
    let mut integer = 0i64;
    let mut sign = 1i64;
    let mut first_digit = true;
    loop {
        let byte = bytes.next();
        if let Some(decoded_byte) = byte {
            match *decoded_byte as char {
                END_TOKEN => break,
                NEGATIVE_SIGN if first_digit => sign = -1i64,
                '0'..='9' => integer = integer * 10 + (decoded_byte - b'0') as i64,
                _ => return Err(BencodeDecoderError::DecodeInt(*decoded_byte)),
            }
        } else {
            return Err(BencodeDecoderError::UnexpectedEndOfStream);
        }
        first_digit = false;
    }
    Ok(sign * integer)
}

pub fn read_string(bytes: &mut slice::Iter<u8>, byte: u8) -> Result<Vec<u8>, BencodeDecoderError> {
    let mut length = byte as usize - ('0' as usize);

    loop {
        let next_byte = bytes.next();
        if let Some(byte) = next_byte {
            let byte = *byte;
            match byte as char {
                STRING_START_TOKEN => break,
                '0'..='9' => length = length * 10 + byte as usize - ('0' as usize),
                _ => return Err(BencodeDecoderError::DecodeInt(byte)),
            }
        } else {
            return Err(BencodeDecoderError::DecodeInt(byte));
        }
    }

    let mut string: Vec<u8> = Vec::with_capacity(length);
    for _ in 0..length {
        match bytes.next() {
            Some(byte) => string.push(*byte),
            None => return Err(BencodeDecoderError::UnexpectedEndOfStream),
        }
    }
    Ok(string)
}

pub fn read_list(
    bytes: &mut slice::Iter<u8>,
) -> Result<Vec<BencodeDecodedValue>, BencodeDecoderError> {
    let mut list: Vec<BencodeDecodedValue> = Vec::new();
    loop {
        let next_item = decode_and_consume_iterator(bytes)?;
        match next_item {
            BencodeDecodedValue::End => break,
            _ => list.push(next_item),
        }
    }
    Ok(list)
}

pub fn read_dictionary(
    bytes: &mut slice::Iter<u8>,
) -> Result<HashMap<Vec<u8>, BencodeDecodedValue>, BencodeDecoderError> {
    let mut dictionary: HashMap<Vec<u8>, BencodeDecodedValue> = HashMap::new();
    loop {
        let next_item = decode_and_consume_iterator(bytes)?;
        match next_item {
            BencodeDecodedValue::End => break,
            BencodeDecodedValue::String(key) => {
                dictionary.insert(key, decode_and_consume_iterator(bytes)?);
            }
            _ => return Err(BencodeDecoderError::UnexpectedDictionaryKey(next_item)),
        }
    }
    Ok(dictionary)
}

fn decode_and_consume_iterator(
    bytes: &mut slice::Iter<u8>,
) -> Result<BencodeDecodedValue, BencodeDecoderError> {
    let next_byte = bytes.next();
    if let Some(byte) = next_byte {
        match *byte as char {
            INTEGER_START_TOKEN => read_integer(bytes).map(BencodeDecodedValue::Integer),
            LIST_START_TOKEN => read_list(bytes).map(BencodeDecodedValue::List),
            '0'..='9' => read_string(bytes, *byte).map(BencodeDecodedValue::String),
            DICTIONARY_START_TOKEN => read_dictionary(bytes).map(BencodeDecodedValue::Dictionary),
            END_TOKEN => Ok(BencodeDecodedValue::End),
            _ => Err(BencodeDecoderError::UnexpectedEndOfStream),
        }
    } else {
        Err(BencodeDecoderError::UnexpectedEndOfStream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_bencode() {
        assert!(matches!(
            decode(b""),
            Err(BencodeDecoderError::UnexpectedEndOfStream)
        ));
    }

    #[test]
    fn decodes_positive_number() {
        assert_eq!(decode(b"i123e").unwrap(), BencodeDecodedValue::Integer(123));
    }

    #[test]
    fn decodes_negative_number() {
        assert_eq!(
            decode(b"i-123e").unwrap(),
            BencodeDecodedValue::Integer(-123)
        );
    }

    #[test]
    fn decode_number_fails_unexpected_end_of_stream() {
        assert!(matches!(
            decode(b"i"),
            Err(BencodeDecoderError::UnexpectedEndOfStream)
        ));
        assert!(matches!(
            decode(b"i123"),
            Err(BencodeDecoderError::UnexpectedEndOfStream)
        ));
    }

    #[test]
    fn decode_number_fails_invalid_digit() {
        assert!(matches!(
            decode(b"i123f"),
            Err(BencodeDecoderError::DecodeInt(b'f'))
        ));
    }

    #[test]
    fn decodes_number_fails_wrong_negative_format() {
        assert!(matches!(
            decode(b"i--123e"),
            Err(BencodeDecoderError::DecodeInt(b'-'))
        ));
        assert!(matches!(
            decode(b"i-12-3e"),
            Err(BencodeDecoderError::DecodeInt(b'-'))
        ));
    }

    #[test]
    fn decodes_empty_string() {
        assert_eq!(decode(b"0:").unwrap(), BencodeDecodedValue::String(vec![]));
    }

    #[test]
    fn decodes_string_with_one_letter() {
        assert_eq!(
            decode(b"1:a").unwrap(),
            BencodeDecodedValue::String(vec![b'a'])
        );
    }

    #[test]
    fn decode_string_with_multiple_letters() {
        assert_eq!(
            decode(b"15:aabaaaaaaaaaaER").unwrap(),
            BencodeDecodedValue::String(b"aabaaaaaaaaaaER".to_vec())
        )
    }
    #[test]
    fn decode_string_reads_only_up_to_length() {
        assert_eq!(
            decode(b"5:aabaaaaaaaaaaER").unwrap(),
            BencodeDecodedValue::String(b"aabaa".to_vec())
        )
    }

    #[test]
    fn decode_string_fails_unexpected_end_of_stream() {
        assert!(matches!(
            decode(b"8:afaw"),
            Err(BencodeDecoderError::UnexpectedEndOfStream)
        ));
    }

    #[test]
    fn decodes_empty_list() {
        assert_eq!(decode(b"le").unwrap(), BencodeDecodedValue::List(vec![]));
    }

    #[test]
    fn decodes_list_with_one_element() {
        assert_eq!(
            decode(b"li123ee").unwrap(),
            BencodeDecodedValue::List(vec![BencodeDecodedValue::Integer(123)])
        );
    }

    #[test]
    fn decodes_list_with_multiple_element() {
        assert_eq!(
            decode(b"li123ei3ei523ee").unwrap(),
            BencodeDecodedValue::List(vec![
                BencodeDecodedValue::Integer(123),
                BencodeDecodedValue::Integer(3),
                BencodeDecodedValue::Integer(523)
            ])
        );
    }
    #[test]
    fn decode_list_fails_unexpected_end_of_stream() {
        assert!(matches!(
            decode(b"li123ei3ei523").unwrap_err(),
            BencodeDecoderError::UnexpectedEndOfStream
        ))
    }

    #[test]
    fn decode_empty_dictionary() {
        assert_eq!(
            decode(b"de").unwrap(),
            BencodeDecodedValue::Dictionary(HashMap::new())
        );
    }

    #[test]
    fn decode_dictionary_with_one_element() {
        assert_eq!(
            decode(b"d1:ai123ee").unwrap(),
            BencodeDecodedValue::Dictionary(HashMap::from([(
                b"a".to_vec(),
                BencodeDecodedValue::Integer(123)
            )]))
        );
    }

    #[test]
    fn decode_dictionary_with_multiple_element() {
        assert_eq!(
            decode(b"d4:holai123e4:chaui321ee").unwrap(),
            BencodeDecodedValue::Dictionary(HashMap::from([
                (b"hola".to_vec(), BencodeDecodedValue::Integer(123)),
                (b"chau".to_vec(), BencodeDecodedValue::Integer(321)),
            ]))
        );
    }

    // complex cases:
    #[test]
    fn decode_list_with_multiple_e() {
        assert_eq!(
            decode(b"l4:eeeee").unwrap(),
            BencodeDecodedValue::List(vec![BencodeDecodedValue::String(b"eeee".to_vec())])
        );
    }

    #[test]
    fn decode_dictionary_with_multiple_e() {
        assert_eq!(
            decode(b"d4:eeee4:eeeee").unwrap(),
            BencodeDecodedValue::Dictionary(HashMap::from([(
                b"eeee".to_vec(),
                BencodeDecodedValue::String(b"eeee".to_vec())
            )]))
        );
    }

    #[test]
    fn decode_list_of_multiple_depths_and_types() {
        assert_eq!(
            decode(b"li100e4:holali20eee").unwrap(),
            BencodeDecodedValue::List(vec![
                BencodeDecodedValue::Integer(100),
                BencodeDecodedValue::String(b"hola".to_vec()),
                BencodeDecodedValue::List(vec![BencodeDecodedValue::Integer(20)])
            ])
        );
    }

    #[test]
    fn decode_dictionary_of_multiple_depths_and_types() {
        assert_eq!(
            decode(b"d1:ai123e4:hola4:chau4:testd1:ai123e4:hola4:chauee").unwrap(),
            BencodeDecodedValue::Dictionary(HashMap::from([
                (b"a".to_vec(), BencodeDecodedValue::Integer(123)),
                (
                    b"hola".to_vec(),
                    BencodeDecodedValue::String(b"chau".to_vec())
                ),
                (
                    b"test".to_vec(),
                    BencodeDecodedValue::Dictionary(HashMap::from([
                        (b"a".to_vec(), BencodeDecodedValue::Integer(123)),
                        (
                            b"hola".to_vec(),
                            BencodeDecodedValue::String(b"chau".to_vec())
                        ),
                    ]))
                )
            ]))
        );
    }
}
