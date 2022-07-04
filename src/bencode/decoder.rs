use super::errors::BencodeDecoderError;
use super::types::BencodeDecodedValue;
use std::collections::HashMap;
const INTEGER_START_TOKEN: char = 'i';
const LIST_START_TOKEN: char = 'l';
const DICTIONARY_START_TOKEN: char = 'd';
const END_TOKEN: char = 'e';
const STRING_START_TOKEN: char = ':';
const NEGATIVE_SIGN: char = '-';
use crate::boxed_result::BoxedResult;

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
    let mut bytes = bytes.iter().enumerate();
    let bencoded_value = decode_and_consume_iterator(&mut bytes)?;
    Ok(bencoded_value)
}

fn decode_and_consume_iterator(
    bytes: &mut std::iter::Enumerate<std::slice::Iter<'_, u8>>,
) -> BoxedResult<BencodeDecodedValue> {
    let next_byte = bytes.next();
    if let Some((idx, byte)) = next_byte {
        match *byte as char {
            INTEGER_START_TOKEN => read_integer(bytes).map(BencodeDecodedValue::Integer),
            LIST_START_TOKEN => read_list(bytes).map(BencodeDecodedValue::List),
            '0'..='9' => read_string(bytes, *byte).map(BencodeDecodedValue::String),
            DICTIONARY_START_TOKEN => read_dictionary(bytes).map(BencodeDecodedValue::Dictionary),
            END_TOKEN => Ok(BencodeDecodedValue::End),
            _ => Err(BencodeDecoderError(format!(
                "Unknown token {} at position {}",
                *byte as char, idx
            ))
            .into()),
        }
    } else {
        Err(BencodeDecoderError("Unexpected end of stream".to_string()).into())
    }
}

fn read_integer(bytes: &mut std::iter::Enumerate<std::slice::Iter<'_, u8>>) -> BoxedResult<i64> {
    let mut integer = 0i64;
    let mut sign = 1i64;
    let mut first_digit = true;
    let mut is_zero = false;
    loop {
        let byte = bytes.next();
        if let Some((idx, decoded_byte)) = byte {
            match *decoded_byte as char {
                END_TOKEN => break,
                NEGATIVE_SIGN if first_digit => sign = -1i64,
                '0' if first_digit => is_zero = true,
                '0'..='9' if !is_zero => integer = integer * 10 + (decoded_byte - b'0') as i64,
                '0'..='9' if is_zero => {
                    return Err(BencodeDecoderError(format!(
                        "Unexpected zero in integer at position {}",
                        idx
                    ))
                    .into())
                }
                _ => {
                    return Err(BencodeDecoderError(format!(
                        "Invalid integer byte {} at position {}",
                        *decoded_byte, idx
                    ))
                    .into())
                }
            }
        } else {
            return Err(BencodeDecoderError(
                "Unexpected end of stream while reading integer".to_string(),
            )
            .into());
        }
        first_digit = false;
    }
    Ok(sign * integer)
}

fn read_string(
    bytes: &mut std::iter::Enumerate<std::slice::Iter<'_, u8>>,
    byte: u8,
) -> BoxedResult<Vec<u8>> {
    let mut length = byte as usize - ('0' as usize);

    loop {
        let next_byte = bytes.next();
        if let Some((idx, byte)) = next_byte {
            let byte = *byte;
            match byte as char {
                STRING_START_TOKEN => break,
                '0'..='9' => length = length * 10 + byte as usize - ('0' as usize),
                _ => {
                    return Err(BencodeDecoderError(format!(
                        "Invalid string length digit {} at position {}",
                        byte as char, idx
                    ))
                    .into())
                }
            }
        } else {
            return Err(BencodeDecoderError(
                "Unexpected end of stream while reading string length".to_string(),
            )
            .into());
        }
    }

    let mut string = vec![];
    for _ in 0..length {
        match bytes.next() {
            Some((_, byte)) => string.push(*byte),
            None => {
                return Err(BencodeDecoderError(
                    "Unexpected end of stream while pushing bytes to string".to_string(),
                )
                .into())
            }
        }
    }
    Ok(string)
}

fn read_list(
    bytes: &mut std::iter::Enumerate<std::slice::Iter<'_, u8>>,
) -> BoxedResult<Vec<BencodeDecodedValue>> {
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

fn read_dictionary(
    bytes: &mut std::iter::Enumerate<std::slice::Iter<'_, u8>>,
) -> BoxedResult<HashMap<Vec<u8>, BencodeDecodedValue>> {
    let mut dictionary: HashMap<Vec<u8>, BencodeDecodedValue> = HashMap::new();
    loop {
        let next_item = decode_and_consume_iterator(bytes)?;
        match next_item {
            BencodeDecodedValue::End => break,
            BencodeDecodedValue::String(key) => {
                dictionary.insert(key, decode_and_consume_iterator(bytes)?);
            }
            invalid_key => {
                return Err(BencodeDecoderError(format!(
                    "Invalid dictionary key {:?}",
                    invalid_key
                ))
                .into())
            }
        }
    }
    Ok(dictionary)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(decode(b"i").is_err());
        assert!(decode(b"i123").is_err());
    }

    #[test]
    fn decode_number_fails_invalid_digit() {
        assert!(decode(b"i123f").is_err());
    }

    #[test]
    fn decode_number_fails_invalid_zeros_on_left() {
        assert!(decode(b"i0123e").is_err());
    }

    #[test]
    fn decodes_number_fails_wrong_negative_format() {
        assert!(decode(b"i--123e").is_err());
        assert!(decode(b"i-12-3e").is_err());
    }

    #[test]
    fn decodes_empty_string() {
        assert_eq!(
            decode(b"0:").unwrap(),
            BencodeDecodedValue::String(b"".to_vec())
        );
    }

    #[test]
    fn decodes_string_with_one_letter() {
        assert_eq!(
            decode(b"1:a").unwrap(),
            BencodeDecodedValue::String(b"a".to_vec())
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
        assert!(decode(b"8:afaw").is_err());
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
        assert!(decode(b"li123ei3ei523").is_err())
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
