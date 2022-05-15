pub use super::super::bencode::*;
use super::super::metainfo::*;
use super::errors::*;
use std::collections::HashMap;

///Receives a byte array and decodes it from Bencode to build a Metainfo Struct.
//To do:Codificar bytes de Hashmap['info'] devuelta a Bencode y despues a SHA-1 (permitieron importar crate en discord) para info_hash de Metainfo
//devuelve metainfo.
pub fn parse_torrent(bytes: &[u8]) -> Result<Metainfo, TorrentParserError> {
    let decoded = decode(bytes).unwrap();
    if let BencodeDecodedValue::Dictionary(hashmap) = decoded {
        return build_metainfo(hashmap).map_err(TorrentParserError::MetainfoBuildError);
    };
    let info = Info {
        piece_length: 164,
        pieces: vec![1, 2, 3],
        name: String::from("nombre"),
        length: 32,
    };
    let metainfo = Metainfo {
        info,
        info_hash: vec![1, 2, 3],
        announce: String::from("Fake URL"),
    };
    Ok(metainfo)

    // match decoded {
    //     BencodeDecodedValue::Dictionary(hashmap) => return build_metainfo(hashmap).map_err(|e:MetainfoBuildError| TorrentParserError::MetainfoBuildError(e)),
    //     _ => return Err(TorrentParserError::UnexpectedBencodeDecodedValue(decoded)) //its supposed to always be a hashmap
    // }
}

fn build_metainfo(
    _hashmap: HashMap<String, BencodeDecodedValue>,
) -> Result<Metainfo, MetainfoBuildError> {
    let info = Info {
        piece_length: 164,
        pieces: vec![1, 2, 3],
        name: String::from("nombre"),
        length: 32,
    };
    let metainfo = Metainfo {
        info,
        info_hash: vec![1, 2, 3],
        announce: String::from("Fake URL"),
    };
    Ok(metainfo)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dummy_metainfo() {
        let test_bytes: Vec<u8> = std::fs::read("ubuntu.torrent").unwrap();
        let metainfo = parse_torrent(&test_bytes).unwrap();
        assert_eq!(metainfo.info_hash, vec![1, 2, 3]);
        assert_eq!(metainfo.announce, String::from("Fake URL"));
        assert_eq!(metainfo.info.piece_length, 164);
    }
}
