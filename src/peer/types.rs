use super::errors::*;
use super::service::*;
use super::utils::bitmap_from_pieces_vector;

#[derive(Clone)]
pub struct PeerState {
    pub chocked: bool,
    pub interested: bool,
}
#[derive(Clone)]
pub struct PeerConnectionState {
    pub client: PeerState,
    pub peer: PeerState,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Bitfield(Vec<u8>);

impl Default for Bitfield {
    fn default() -> Self {
        Self::new()
    }
}

impl Bitfield {
    pub fn new() -> Self {
        Bitfield(vec![])
    }

    pub fn non_empty(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<u8> {
        self.0.iter()
    }

    pub fn set_bitfield(&mut self, bitfield: &[u8]) {
        self.0 = bitfield.to_vec();
    }

    pub fn has_piece(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;
        if byte_index >= self.0.len() {
            return false;
        }
        (self.0[byte_index] >> (7 - offset) & 1) != 0
    }

    fn _set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;

        if byte_index >= self.0.len() {
            return;
        }
        self.0[byte_index] |= 1 << (7 - offset);
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
    pub peer_id: Vec<u8>,
    pub peer_message_service_provider:
        fn(
            ip: String,
            port: u16,
        ) -> Result<Box<dyn IClientPeerMessageService + Send>, PeerConnectionError>,
}

impl Peer {
    pub fn connect(
        &self,
    ) -> Result<Box<dyn IClientPeerMessageService + Send>, PeerConnectionError> {
        (self.peer_message_service_provider)(self.ip.clone(), self.port)
    }
}

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

#[allow(dead_code)]
#[derive(Debug, Clone)]
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

    // function tan conver a u32 into 4 bytes vector big endian
    fn u32_to_vec_be(num: u32) -> Vec<u8> {
        let mut bytes = vec![0; 4];
        bytes[0] = (num >> 24) as u8;
        bytes[1] = (num >> 16) as u8;
        bytes[2] = (num >> 8) as u8;
        bytes[3] = num as u8;
        bytes
    }

    pub fn request(index: u32, begin: u32, length: u32) -> PeerMessage {
        let mut payload = vec![];
        // write index as 4 bytes to payload
        payload.extend_from_slice(&Self::u32_to_vec_be(index));
        payload.extend_from_slice(&Self::u32_to_vec_be(begin));
        payload.extend_from_slice(&Self::u32_to_vec_be(length));

        PeerMessage {
            id: PeerMessageId::Request,
            length: (payload.len() + 1) as u32,
            payload,
        }
    }

    pub fn piece(piece_index: usize, offset: usize, block: Vec<u8>) -> PeerMessage {
        let mut payload = vec![];
        payload.extend_from_slice(&Self::u32_to_vec_be(piece_index as u32));
        payload.extend_from_slice(&Self::u32_to_vec_be(offset as u32));
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

    pub fn bitfield(pieces: Vec<bool>) -> PeerMessage {
        PeerMessage {
            id: PeerMessageId::Bitfield,
            length: 0,
            payload: bitmap_from_pieces_vector(&pieces),
        }
    }

    pub fn not_intersted() -> PeerMessage {
        PeerMessage {
            id: PeerMessageId::NotInterested,
            length: 0,
            payload: vec![],
        }
    }

    pub fn choke() -> PeerMessage {
        PeerMessage {
            id: PeerMessageId::Choke,
            length: 0,
            payload: vec![],
        }
    }
}
