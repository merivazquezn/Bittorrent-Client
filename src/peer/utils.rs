use super::constants::*;
use crate::metainfo::Metainfo;
use log::*;
use sha1::{Digest, Sha1};

// Gets sha1 hash of vector u8
pub fn sha1_of(vec: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(vec);
    hasher.finalize().to_vec()
}

// gets the u32 value of a big endian vector
pub fn vec_be_to_u32(bytes: &[u8]) -> u32 {
    let mut num = 0;
    for (i, byte) in bytes.iter().enumerate().take(4) {
        num += (*byte as u32) << (8 * (3 - i));
    }
    num
}

// Validates a piece by comparing its 20 byte sha1 values, to the one found in the pieces field of the info dictionary.
// To access value of the info dictionary, we use the piece index.
pub fn valid_piece(piece: &[u8], piece_index: u32, metainfo: &Metainfo) -> bool {
    let real_piece_sha1 = metainfo.info.pieces[piece_index as usize].to_vec();
    let recieved_piece_sha1 = sha1_of(piece);
    info!("Comparing downloaded piece with expected piece");
    info!("Expected piece SHA1: {:?}\n", real_piece_sha1);
    info!("Received piece SHA1: {:?}\n", recieved_piece_sha1);
    recieved_piece_sha1 == real_piece_sha1
}

// Checks if payloads first 4 bytes are equal to the piece index requested, and the next 4 are equal to the offset
// If true then the block recieved is valid
pub fn valid_block(payload: &[u8], requested_index: u32, requested_offset: u32) -> bool {
    let piece_index_recieved = vec_be_to_u32(&payload[0..=3]);
    let offset_recieved = vec_be_to_u32(&payload[4..=7]);
    (piece_index_recieved == requested_index) && (offset_recieved == requested_offset)
}

pub fn is_keep_alive_message(message_length: u32) -> bool {
    message_length == 0
}

pub fn create_handshake_message(info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
    let mut handshake_message = Vec::new();
    handshake_message.extend_from_slice(&[PSTRLEN]);
    handshake_message.extend_from_slice(b"BitTorrent protocol");
    handshake_message.extend_from_slice(&[0u8; 8]);
    handshake_message.extend_from_slice(info_hash);
    handshake_message.extend_from_slice(peer_id);
    handshake_message
}

fn reverse_byte(byte: u8) -> u8 {
    let mut reversed_byte = 0;
    for i in 0..8 {
        reversed_byte |= ((byte >> i) & 1) << (7 - i);
    }
    reversed_byte
}

pub fn bitmap_from_pieces_vector(vector: &[bool]) -> Vec<u8> {
    let mut byte_vec = Vec::new();
    let mut current_byte = 0;
    let mut current_bit = 0;
    for bit in vector {
        if *bit {
            current_byte |= 1 << current_bit;
        }
        current_bit += 1;
        if current_bit == 8 {
            byte_vec.push(current_byte);
            current_bit = 0;
            current_byte = 0;
        }
    }
    if current_bit != 0 {
        byte_vec.push(current_byte);
    }

    let mut bitmap: Vec<u8> = Vec::new();
    for byte in byte_vec {
        bitmap.push(reverse_byte(byte));
    }

    bitmap
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn create_bitmap_from_vector_of_booleans_only_last_piece_is_present() {
        let mut vector = vec![true, false, false, false, false, false, false, false];
        let bitmap = bitmap_from_pieces_vector(&mut vector);
        assert_eq!(bitmap, vec![0b1000_0000]);
    }

    #[test]
    fn create_bitmap_from_vector_of_booleans_interleaved() {
        let mut vector = vec![true, false, true, false, true, false, false, false];
        let bitmap = bitmap_from_pieces_vector(&mut vector);
        assert_eq!(bitmap, vec![0b10101000]);
    }

    #[test]
    fn create_bitmap_from_vector_of_booleans_none_multiple() {
        let mut vector = vec![
            true, false, false, false, false, false, false, false, false, false, false, false,
            false, false, true,
        ];
        let bitmap = bitmap_from_pieces_vector(&mut vector);
        assert_eq!(bitmap, vec![0b1000_0000, 0b0000_0010]);
    }
}
