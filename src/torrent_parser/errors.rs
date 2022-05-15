pub use super::super::bencode::*;

#[derive(Debug)]
///The error type that is returned by the torrent parser (parse_torrent)
pub enum TorrentParserError {
    //There was an error while decoding the file
    DecodeError(BencodeDecoderError),
    ///The torrent file parsing resulted in an unexpected structure (not a Dictionary)
    UnexpectedBencodeDecodedValue(BencodeDecodedValue),
    //There was a problem building the Metainfo struct
    MetainfoBuildError(MetainfoBuildError),
}

#[derive(Debug)]
///The error type that is returned by the Metainfo builder (build_metainfo)
pub enum MetainfoBuildError {
    ///Could not find a necessary key for building the Metainfo in hashmap
    KeyError,
}
