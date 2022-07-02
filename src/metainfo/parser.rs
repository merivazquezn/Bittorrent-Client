pub use super::super::bencode::*;
use super::super::metainfo::*;
use super::errors::*;
use crate::client::SHA1_LENGTH;
use crate::logger::CustomLogger;
use log::*;
use sha1::{Digest, Sha1};
use std::collections::HashMap;
use std::str::from_utf8;
const LOGGER: CustomLogger = CustomLogger::init("Config");
///Receives a byte array and Bencode-Decodes it to build a [Metainfo].
/// ## Example
///
/// ```
/// use std::fs::read;
/// use bittorrent_rustico::metainfo::*;
///
/// let metainfo_bytes = read("sample.torrent").unwrap();
/// let parsed:Result<Metainfo,MetainfoParserError> = parse(&metainfo_bytes);
///
/// ```
pub fn parse(bytes: &[u8]) -> Result<Metainfo, MetainfoParserError> {
    trace!("Decoding bencode bytes");
    let decoded = decode(bytes)
        .map_err(|e| MetainfoParserError::BencodeError(format!("Error decoding bytes: {}", e)))?;
    trace!("Building metainfo");
    build_metainfo(decoded.get_as_dictionary()?)
}

//Builds Metainfo Struct from a hashmap containing the relevant Bencode-Decoded Values
fn build_metainfo(
    hashmap: &HashMap<Vec<u8>, BencodeDecodedValue>,
) -> Result<Metainfo, MetainfoParserError> {
    let info_key = b"info";
    let piece_length_key = b"piece length";
    let pieces_key = b"pieces";
    let name_key = b"name";
    let length_key = b"length";
    let announce_key = b"announce";
    let files_key = b"files";
    let path_key = b"path";

    let info_hashmap_decoded = get_from_bencoded_values_hashmap(hashmap, info_key)?;
    let info_hashmap = info_hashmap_decoded.get_as_dictionary()?;

    let total_length = match get_from_bencoded_values_hashmap(info_hashmap, length_key) {
        Ok(length) => *length.get_as_integer()? as u64,
        Err(_) => {
            let files_hashmap_decoded = get_from_bencoded_values_hashmap(info_hashmap, files_key)?;
            let files = files_hashmap_decoded.get_as_list()?;
            let mut total_length = 0;
            for file in files {
                let file_hashmap = file.get_as_dictionary()?;
                let length = get_from_bencoded_values_hashmap(file_hashmap, length_key)?;
                total_length += length.get_as_integer()?;
            }
            total_length as u64
        }
    };

    let pieces_as_vec_u8 = get_from_bencoded_values_hashmap(info_hashmap, pieces_key)?
        .get_as_string()?
        .to_vec();

    let files: Option<Vec<File>> = match get_from_bencoded_values_hashmap(info_hashmap, files_key) {
        Ok(files_bencoded) => {
            let file_list = files_bencoded.get_as_list()?;
            let mut files = Vec::new();
            for file in file_list {
                let file_hashmap = file.get_as_dictionary()?;
                let path = bencode_list_to_string_path(&get_from_bencoded_values_hashmap(
                    file_hashmap,
                    path_key,
                )?)?;
                let length = *get_from_bencoded_values_hashmap(file_hashmap, length_key)?
                    .get_as_integer()? as u64;
                files.push(File { path, length });
            }
            Some(files)
        }
        Err(_) => None,
    };

    let info = Info {
        piece_length: *get_from_bencoded_values_hashmap(info_hashmap, piece_length_key)?
            .get_as_integer()? as u32,
        pieces: get_vec_of_hashes(&pieces_as_vec_u8),
        name: bencode_decoded_bytes_to_string(info_hashmap, name_key)?,
        length: total_length,
        files,
    };

    let metainfo = Metainfo {
        info,
        info_hash: get_hash(hashmap, info_key),
        announce: bencode_decoded_bytes_to_string(hashmap, announce_key)?,
    };
    validate(&metainfo)?;
    Ok(metainfo)

    // if info_hashmap.contains_key(&files_key.to_vec()) {
    //     // let files_list_bencode = get_from_bencoded_values_hashmap(&info_hashmap, files_key)?;
    //     // let files_list = files_list_bencode.get_as_list()?;

    //     let info = Info {
    //         piece_length: *get_from_bencoded_values_hashmap(info_hashmap, piece_length_key)?
    //             .get_as_integer()? as u32,
    //         pieces: get_vec_of_hashes(&pieces_as_vec_u8),
    //         name: bencode_decoded_bytes_to_string(info_hashmap, name_key)?,
    //         length: total_length,
    //         files: None,
    //     };
    //     let metainfo = Metainfo {
    //         info,
    //         info_hash: get_hash(hashmap, info_key),
    //         announce: bencode_decoded_bytes_to_string(hashmap, announce_key)?,
    //     };
    //     validate(&metainfo)?;
    //     Ok(metainfo)
    // } else {
    //     let info = Info {
    //         piece_length: *get_from_bencoded_values_hashmap(info_hashmap, piece_length_key)?
    //             .get_as_integer()? as u32,
    //         pieces: get_vec_of_hashes(&pieces_as_vec_u8),
    //         name: bencode_decoded_bytes_to_string(info_hashmap, name_key)?,
    //         length: total_length,
    //         files: None,
    //     };
    //     let metainfo = Metainfo {
    //         info,
    //         info_hash: get_hash(hashmap, info_key),
    //         announce: bencode_decoded_bytes_to_string(hashmap, announce_key)?,
    //     };
    //     validate(&metainfo)?;
    //     Ok(metainfo)
    // }
}

// function that converts a Bencoded decoded List and turns it into a Bencode Decoded String
fn bencode_list_to_string_path(list: &BencodeDecodedValue) -> Result<String, BencodeDecoderError> {
    let mut path = String::new();
    let mut list_iter = list.get_as_list()?.iter();
    while let Some(value) = list_iter.next() {
        let value_string = value.get_as_string()?;
        path.push_str(&String::from_utf8_lossy(value_string));
        if list_iter.next().is_some() {
            path.push('/');
        }
    }
    Ok(path)
}

// Converts the vector of pieces into a vector of each piece hash
// each index represent each piece of file
fn get_vec_of_hashes(pieces: &[u8]) -> Vec<Vec<u8>> {
    let mut pieces_as_vec_of_hashes: Vec<Vec<u8>> = vec![];
    let mut index = 0;
    while index < pieces.len() {
        let piece = pieces[index..index + SHA1_LENGTH].to_vec();
        pieces_as_vec_of_hashes.push(piece);
        index += SHA1_LENGTH;
    }
    pieces_as_vec_of_hashes
}

//Retrieves the 20-byte SHA-1 hash from the received hashmap value corresponding to the key
fn get_hash(hashmap: &HashMap<Vec<u8>, BencodeDecodedValue>, key: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    let info = hashmap.get(key).unwrap();
    hasher.update(encode(info));
    let result = hasher.finalize();
    result[..].to_vec()
}

//Returns a Bencode-Decoded Value associated with the key in the received HashMap
fn get_from_bencoded_values_hashmap(
    hashmap: &HashMap<Vec<u8>, BencodeDecodedValue>,
    key: &[u8],
) -> Result<BencodeDecodedValue, MetainfoParserError> {
    let value = hashmap.get(key).ok_or_else(|| {
        MetainfoParserError::MetainfoKeyNotFound(
            String::from_utf8(key.to_vec()).unwrap_or_else(|_| "non-utf8".to_string()),
        )
    })?;
    Ok(value.clone())
}

//Returns a String casted from Vec<u8> found in a hashmap that contains Bencode-Decoded Value
fn bencode_decoded_bytes_to_string(
    hashmap: &HashMap<Vec<u8>, BencodeDecodedValue>,
    key: &[u8],
) -> Result<String, MetainfoParserError> {
    let value_bytes_decoded = get_from_bencoded_values_hashmap(hashmap, key)?;
    let value_bytes = value_bytes_decoded.get_as_string()?;
    let value: &str = from_utf8(value_bytes).map_err(|_| MetainfoParserError::UTF8Error)?;
    Ok(value.to_string())
}

fn validate_pieces(
    pieces: &[Vec<u8>],
    file_length: usize,
    piece_length: usize,
) -> Result<(), MetainfoParserError> {
    if pieces.len() != (file_length + (piece_length - 1)) / piece_length {
        return Err(MetainfoParserError::ValidationError);
    }

    for piece in pieces {
        if piece.len() != SHA1_LENGTH {
            return Err(MetainfoParserError::ValidationError);
        }
    }

    Ok(())
}

//Performs basic validation of certain values in Info and Metainfo
fn validate(metainfo: &Metainfo) -> Result<(), MetainfoParserError> {
    let info: &Info = &metainfo.info;
    if metainfo.announce.is_empty()
        || info.piece_length == 0
        || info.pieces.is_empty()
        || info.length == 0
    {
        return Err(MetainfoParserError::ValidationError);
    }
    validate_pieces(
        &info.pieces,
        info.length as usize,
        info.piece_length as usize,
    )?;
    LOGGER.info_str("Torrent parsed successfully");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decode_hex(hex: &str) -> Result<Vec<u8>, String> {
        let mut bytes = Vec::new();
        for i in 0..hex.len() / 2 {
            let s = &hex[i * 2..i * 2 + 2];
            let byte =
                u8::from_str_radix(s, 16).map_err(|_| format!("Invalid hex string: {}", s))?;
            bytes.push(byte);
        }
        Ok(bytes)
    }

    #[test]
    fn sample_metainfo() {
        let test_bytes: Vec<u8> = std::fs::read("sample.torrent").unwrap();
        let metainfo = parse(&test_bytes).unwrap();

        let mut pieces: Vec<Vec<u8>> = Vec::new();
        pieces.push(vec![
            92, 197, 230, 82, 190, 13, 230, 242, 120, 5, 179, 4, 100, 255, 155, 0, 244, 137, 240,
            201,
        ]);

        let expected_info: Info = Info {
            piece_length: 65536,
            pieces: pieces,
            name: "sample.txt".to_string(),
            length: 20,
            files: None,
        };

        let expected_metainfo: Metainfo = Metainfo {
            info: expected_info,
            info_hash: decode_hex("d0d14c926e6e99761a2fdcff27b403d96376eff6").unwrap(),
            announce: "udp://tracker.openbittorrent.com:80".to_string(),
        };

        assert_eq!(metainfo, expected_metainfo);
    }

    #[test]
    fn works_on_ubuntu_torrent() {
        let test_bytes: Vec<u8> = std::fs::read("./example_torrents/ubuntu.torrent").unwrap();
        let metainfo_result = parse(&test_bytes);
        assert!(matches!(metainfo_result, Ok(_)));
    }

    #[test]
    fn empty_byte_array() {
        let empty_bytes: Vec<u8> = Vec::new();
        let result = parse(&empty_bytes);
        assert!(matches!(
            result.unwrap_err(),
            MetainfoParserError::BencodeError(_)
        ))
    }

    #[test]
    fn invalid_byte_array() {
        let invalid_bytes: Vec<u8> = b"CantMakeAMetainfoOutOfThis".to_vec();
        let result = parse(&invalid_bytes);
        assert!(matches!(
            result.unwrap_err(),
            MetainfoParserError::BencodeError(_)
        ))
    }

    #[test]
    fn necessary_key_not_dictionary() {
        let invalid_bytes: Vec<u8> = b"d3:cow3:moo4:spam4:eggse".to_vec();
        let result = parse(&invalid_bytes);
        assert!(matches!(
            result.unwrap_err(),
            MetainfoParserError::MetainfoKeyNotFound(_)
        ))
    }

    #[test]
    fn unexpected_value_type() {
        let invalid_bytes: Vec<u8> = b"d3:cow3:moo4:info4:eggse".to_vec();
        let result = parse(&invalid_bytes);
        assert!(matches!(
            result.unwrap_err(),
            MetainfoParserError::BencodeError(_)
        ))
    }

    #[test]
    fn invalid_values() {
        let invalid_info: Info = Info {
            piece_length: 65536,
            pieces: vec![vec![1, 2], vec![1, 3], vec![4, 0]], //array length of the first piece hash is not a multiple of 20!
            name: "sample.txt".to_string(),
            length: 20,
            files: None,
        };

        let invalid_metainfo: Metainfo = Metainfo {
            info: invalid_info,
            info_hash: decode_hex("d0d14c926e6e99761a2fdcff27b403d96376eff6").unwrap(),
            announce: "udp://tracker.openbittorrent.com:80".to_string(),
        };

        assert!(matches!(
            validate(&invalid_metainfo).unwrap_err(),
            MetainfoParserError::ValidationError
        ))
    }
}
