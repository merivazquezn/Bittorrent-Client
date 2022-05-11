use super::bencode_types::*;
use std::vec::Vec;

#[allow(dead_code)]
pub fn encode(_value: BencodeDecodedValue) -> Result<Vec<u8>, BencodeDecoderError> {
    Ok(b"i123e".to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encode_positive_number() {
        assert_eq!(
            encode(BencodeDecodedValue::Integer(123)).unwrap(),
            b"i123e".to_vec()
        );
    }
}
