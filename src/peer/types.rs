use log::*;
use std::io::{Read, Write};
use std::net::TcpStream;

const PSTRLEN: u8 = 19;
const HANDSHAKE_LENGTH: usize = 68;

// Message constants
const MESSAGE_ID_SIZE: usize = 1;
const MESSAGE_LENGTH_SIZE: usize = 4;

#[allow(dead_code)]
struct Bitfield(Vec<u8>);

impl Bitfield {
    #[allow(dead_code)]
    fn has_piece(&self, index: usize) -> bool {
        let byte_index = index / 8;
        let offset = index % 8;
        if byte_index >= self.0.len() {
            return false;
        }
        (self.0[byte_index] >> (7 - offset) & 1) != 0
    }

    #[allow(dead_code)]
    fn set_piece(&mut self, index: usize) {
        let byte_index = index / 8;
        let offset = index % 8;

        if byte_index >= self.0.len() {
            return;
        }
        self.0[byte_index] |= 1 << (7 - offset);
    }
}

#[derive(Debug, PartialEq)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
    pub peer_id: Vec<u8>,
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
}

impl PeerMessageId {
    fn from_u8(id: u8) -> Result<PeerMessageId, String> {
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
}

pub struct PeerMessageStream {
    stream: TcpStream,
}

impl PeerMessageStream {
    pub fn connect_to_peer(peer: &Peer) -> Result<Self, Box<dyn std::error::Error>> {
        let stream = TcpStream::connect(format!("{}:{}", peer.ip, peer.port)).unwrap();
        Ok(Self { stream })
    }

    fn create_handshake_message(&self, info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
        let mut handshake_message = Vec::new();
        handshake_message.extend_from_slice(&[PSTRLEN]);
        handshake_message.extend_from_slice(b"BitTorrent protocol");
        handshake_message.extend_from_slice(&[0u8; 8]);
        handshake_message.extend_from_slice(info_hash);
        handshake_message.extend_from_slice(peer_id);
        handshake_message
    }
}

impl PeerMessageService for PeerMessageStream {
    fn wait_for_message(&mut self) -> Result<PeerMessage, Box<dyn std::error::Error>> {
        let mut message_length = [0u8; MESSAGE_LENGTH_SIZE];
        self.stream.read_exact(&mut message_length).unwrap();
        let message_length = u32::from_be_bytes(message_length);
        let mut message_id = [0u8; MESSAGE_ID_SIZE];
        self.stream.read_exact(&mut message_id).unwrap();
        let mut payload: Vec<u8> = vec![0; (message_length - 1) as usize];
        self.stream.read_exact(&mut payload).unwrap();

        let msg = PeerMessage {
            id: PeerMessageId::from_u8(message_id[0])?,
            length: message_length,
            payload,
        };
        debug!("message received: {:?}", msg.id);
        Ok(msg)
    }

    fn handshake(
        &mut self,
        info_hash: &[u8],
        peer_id: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let handshake_message = self.create_handshake_message(info_hash, peer_id);
        self.stream.write_all(&handshake_message).unwrap();
        let mut handshake_response = [0u8; HANDSHAKE_LENGTH];
        self.stream.read_exact(&mut handshake_response).unwrap();
        debug!("handshake successful");
        // todo fijarse que pasa si el handshake no es correcto
        Ok(())
    }

    fn send_message(&mut self, message: &PeerMessage) -> Result<(), Box<dyn std::error::Error>> {
        let mut bytes = Vec::with_capacity((message.length + 4) as usize);
        bytes.extend_from_slice(&message.length.to_be_bytes());
        bytes.extend_from_slice(&(message.id as u8).to_be_bytes());
        bytes.extend_from_slice(&message.payload);
        self.stream.write_all(&bytes).unwrap();
        debug!("message sent: {:?}", message);
        Ok(())
    }
}

pub trait PeerMessageService {
    fn handshake(
        &mut self,
        info_hash: &[u8],
        peer_id: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>>;
    fn wait_for_message(&mut self) -> Result<PeerMessage, Box<dyn std::error::Error>>;
    fn send_message(&mut self, message: &PeerMessage) -> Result<(), Box<dyn std::error::Error>>;
}
