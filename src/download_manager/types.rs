/// Represents a Piece to download or to share to other peers
pub struct Piece {
    /// The piece number representing the index on the pieces array in Metainfo
    pub piece_number: u32,

    /// To be filled with the downloaded data
    pub data: Vec<u8>,

    /// The size of the piece in bytes
    // Note: Kind of redundant because it would be the same in every piece except the last one,
    // but migth be useful to store it somewhere
    // TODO: Check if needs to be removed
    pub size_in_bytes: u64,
}

impl Piece {
    /// Creates a new Piece from its number and size
    pub fn new(piece_number: u32, size_in_bytes: u64) -> Self {
        let empty_data = Vec::new();
        Piece {
            piece_number,
            data: empty_data,
            size_in_bytes,
        }
    }
}
