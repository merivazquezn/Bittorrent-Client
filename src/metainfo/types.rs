use super::errors::MetainfoParserError;
use super::parser::parse;
use std::fs;
use std::vec::Vec;

#[derive(Debug)]
///Bencode-Decoded metainfo file.
pub struct Metainfo {
    ///contains information about the file to download
    pub info: Info,
    ///20 byte SHA-1 hash obtained from hashing 'info' dictionary
    pub info_hash: Vec<u8>,
    ///the announce URL used for connecting to the tracker
    pub announce: String,
}
#[derive(Debug)]
///Bencode-Decoded Info Dictionary of a metainfo file.
pub struct Info {
    ///the length in bytes of each single piece
    pub piece_length: u32,
    ///the concatenation of the 20 byte SHA-1 hashes of all pieces to verify the data sent to us by peers
    pub pieces: Vec<u8>,
    ///the file name
    pub name: String,
    ///the length in bytes of the file to download
    pub length: u64,
}

impl Metainfo {
    pub fn from_torrent(torrent_path: &str) -> Result<Metainfo, MetainfoParserError> {
        let torrent_bytes: Vec<u8> = fs::read(torrent_path)?;
        parse(&torrent_bytes)
    }
}

impl PartialEq for Info {
    fn eq(&self, other: &Self) -> bool {
        self.piece_length == other.piece_length
            && self.pieces == other.pieces
            && self.name == other.name
            && self.length == other.length
    }
}

impl PartialEq for Metainfo {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info
            && self.info_hash == other.info_hash
            && self.announce == other.announce
    }
}
