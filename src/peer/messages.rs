use super::utils::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PeerMessageId {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
    Port,
    KeepAlive,
}

impl PeerMessageId {
    pub fn from_u8(id: u8) -> Result<PeerMessageId, String> {
        match id {
            0 => Ok(PeerMessageId::Choke),
            1 => Ok(PeerMessageId::Unchoke),
            2 => Ok(PeerMessageId::Interested),
            3 => Ok(PeerMessageId::NotInterested),
            4 => Ok(PeerMessageId::Have),
            5 => Ok(PeerMessageId::Bitfield),
            6 => Ok(PeerMessageId::Request),
            7 => Ok(PeerMessageId::Piece),
            8 => Ok(PeerMessageId::Cancel),
            9 => Ok(PeerMessageId::Port),
            _ => Err(format!("Invalid message id: {}", id)),
        }
    }
}

#[derive(Debug)]
pub struct PeerMessage {
    pub id: PeerMessageId,
    pub length: u32,
    pub payload: Vec<u8>,
}

impl PeerMessage {
    pub fn unchoke() -> PeerMessage {
        const UNCHOKE_MSG_LENGTH: u32 = 1;
        PeerMessage {
            id: PeerMessageId::Unchoke,
            length: UNCHOKE_MSG_LENGTH,
            payload: vec![],
        }
    }
    pub fn interested() -> PeerMessage {
        const INTERESTED_MSG_LENGTH: u32 = 1;
        PeerMessage {
            id: PeerMessageId::Interested,
            length: INTERESTED_MSG_LENGTH,
            payload: vec![],
        }
    }

    pub fn request(index: u32, begin: u32, length: u32) -> PeerMessage {
        let mut payload = vec![];
        // write index as 4 bytes to payload
        payload.extend_from_slice(&u32_to_vec_be(index));
        payload.extend_from_slice(&u32_to_vec_be(begin));
        payload.extend_from_slice(&u32_to_vec_be(length));

        PeerMessage {
            id: PeerMessageId::Request,
            length: (payload.len() + 1) as u32,
            payload,
        }
    }

    pub fn piece(piece_index: u32, offset: u32, block: Vec<u8>) -> PeerMessage {
        let mut payload = vec![];
        payload.extend_from_slice(&u32_to_vec_be(piece_index));
        payload.extend_from_slice(&u32_to_vec_be(offset));
        payload.extend_from_slice(&block);

        PeerMessage {
            id: PeerMessageId::Piece,
            length: (payload.len() + 1) as u32,
            payload,
        }
    }

    pub fn keep_alive() -> PeerMessage {
        PeerMessage {
            id: PeerMessageId::Choke,
            length: 0,
            payload: vec![],
        }
    }
}
