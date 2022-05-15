use std::vec::Vec;

///Bencode-Decoded Metainfo about a torrent file.
pub struct Metainfo {
    ///information about the file to download
    pub info: Info,
    ///URL-encoded 20 byte SHA-1 hash
    pub info_hash: Vec<u8>,
    ///the announce URL to use for connecting to the tracker
    pub announce: String,
}

///Info Dictionary of a torrent file.
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
