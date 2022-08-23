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

pub fn payload_from_request_message(request: RequestMessage) -> Vec<u8> {
    let mut payload: Vec<u8> = Vec::new();
    payload.extend_from_slice(&(request.index as u32).to_be_bytes());
    payload.extend_from_slice(&(request.begin as u32).to_be_bytes());
    payload.extend_from_slice(&(request.length as u32).to_be_bytes());
    payload
}

pub fn client_has_piece(piece_index: usize, pieces_dir: &str) -> bool {
    let piece_path = format!("{}/{}", pieces_dir, piece_index);
    println!("piece path: {:?}", piece_path);
    Path::new(&piece_path).exists()
}

pub fn read_piece(piece_path: &str) -> Result<Vec<u8>, ServerError> {
    let mut file = std::fs::File::open(piece_path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
}

pub fn get_block_from_piece(
    piece_data: Vec<u8>,
    begin: usize,
    length: usize,
) -> Result<Vec<u8>, ServerError> {
    // if piece_data.len() < begin + length {
    //     error!(
    //         "Piece data is too short to get the block.\n Piece data: {:?}\nbegin: {}\nlength: {}",
    //         piece_data, begin, length
    //     );
    //     return Err(ServerError::PieceRequestError(
    //         "Piece data is too short to contain the requested block".to_string(),
    //     ));
    // }
    if begin + length > piece_data.len() {
        return Ok(piece_data[begin..].to_vec().to_vec());
    }
    Ok(piece_data[begin..(begin + length)].to_vec())
}

pub fn get_block_index(begin: usize, block_size: usize) -> usize {
    begin / block_size
}

pub fn get_pieces_vector(piece_count: usize, download_path: &str) -> Vec<bool> {
    let mut piece_vector: Vec<bool> = Vec::new();
    for i in 0..piece_count {
        piece_vector.push(client_has_piece(i, download_path));
    }
    println!("pieces vector: {:?}", piece_vector);
    piece_vector
}
