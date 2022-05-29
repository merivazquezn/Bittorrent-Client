/// Represents a Piece to download or to share to other peers
pub struct Piece {
    /// The piece number representing the index on the pieces array in Metainfo
    pub piece_number: u32,

    /// To be filled with the downloaded data
    pub data: Vec<u8>,
}
