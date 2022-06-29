use super::errors::MetainfoParserError;
use super::parser::parse;
use crate::logger::CustomLogger;
use log::*;
use std::fs;
use std::vec::Vec;
const LOGGER: CustomLogger = CustomLogger::init("Config");

#[derive(Debug, Clone)]
///Bencode-Decoded metainfo file.
pub struct Metainfo {
    ///contains information about the file to download
    pub info: Info,
    ///20 byte SHA-1 hash obtained from hashing 'info' dictionary
    pub info_hash: Vec<u8>,
    ///the announce URL used for connecting to the tracker
    pub announce: String,
}
#[derive(Debug, Clone)]
///Bencode-Decoded Info Dictionary of a metainfo file.
pub struct Info {
    ///the length in bytes of each single piece
    pub piece_length: u32,
    ///the concatenation of the 20 byte SHA-1 hashes of all pieces to verify the data sent to us by peers
    pub pieces: Vec<Vec<u8>>,
    ///the file name
    pub name: String,
    ///the size of the torrent in bytes
    pub length: u64,
    /// files structure in case it is a multi-file torrent
    pub files: Option<Vec<File>>,
}

#[derive(Debug, Clone)]
pub struct File {
    pub path: String,
    pub length: u64,
}

impl Metainfo {
    pub fn from_torrent(torrent_path: &str) -> Result<Metainfo, MetainfoParserError> {
        LOGGER.info(format!("reading torrent file from path: {}", torrent_path));
        let torrent_bytes: Vec<u8> = fs::read(torrent_path)?;
        debug!("Parsing torrent file");
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
