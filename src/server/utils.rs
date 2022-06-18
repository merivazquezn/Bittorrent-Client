use super::RequestMessage;
use super::ServerError;
use std::io::Read;
use std::path::Path;

pub fn request_from_payload(payload: Vec<u8>) -> Result<RequestMessage, ServerError> {
    let error_message = "Invalid payload on request message".to_string();
    if payload.len() != 12 {
        return Err(ServerError::PieceRequestError(error_message));
    }

    let index_bytes = payload[0..4]
        .try_into()
        .map_err(|_| ServerError::PieceRequestError(error_message.clone()))?;
    let begin_bytes = payload[4..8]
        .try_into()
        .map_err(|_| ServerError::PieceRequestError(error_message.clone()))?;
    let length_bytes = payload[8..12]
        .try_into()
        .map_err(|_| ServerError::PieceRequestError(error_message.clone()))?;

    Ok(RequestMessage {
        index: u32::from_be_bytes(index_bytes) as usize,
        begin: u32::from_be_bytes(begin_bytes) as usize,
        length: u32::from_be_bytes(length_bytes) as usize,
    })
}

pub fn client_has_piece(piece_index: usize, pieces_dir: &str) -> bool {
    let piece_path = format!("{}/{}", pieces_dir, piece_index);
    Path::new(&piece_path).exists()
}

pub fn read_piece(piece_path: &str) -> Result<Vec<u8>, ServerError> {
    let mut file = std::fs::File::open(piece_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

pub fn get_block_from_piece(piece_data: Vec<u8>, begin: usize, length: usize) -> Vec<u8> {
    piece_data[begin..length].to_vec()
}
