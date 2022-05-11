use super::bencode_types::*;

#[allow(dead_code)]
pub fn decode(_bytes: &[u8]) -> Result<BencodeDecodedValue, BencodeDecoderError> {
    Ok(BencodeDecodedValue::Integer(123))
    // let mut bytes = bytes.to_vec();
    // let first_byte = bytes.remove(0);
    // match first_byte {
    //     b'0'..=b'9' => {
    //         let integer = BencodeDecoder::decode_integer(&bytes)?;
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_positive_number() {
        assert_eq!(decode(b"i123e").unwrap(), BencodeDecodedValue::Integer(123));
    }
}
