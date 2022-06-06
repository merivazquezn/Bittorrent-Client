use super::constants::*;
use super::errors::*;
use super::utils::is_keep_alive_message;
use super::IPeerMessageServiceError;
use crate::boxed_result::BoxedResult;
use log::*;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const PSTRLEN: u8 = 19;
const HANDSHAKE_LENGTH: usize = 68;

// Message constants
const MESSAGE_ID_SIZE: usize = 1;
const MESSAGE_LENGTH_SIZE: usize = 4;

#[allow(dead_code)]
pub struct Bitfield(Vec<u8>);

impl Bitfield {
    pub fn new() -> Self {
        Bitfield(vec![])
    }

    pub fn non_empty(&self) -> bool {
        !self.0.is_empty()
    }

    pub fn set_bitfield(&mut self, bitfield: &[u8]) {
        self.0 = bitfield.to_vec();
    }

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
    KeepAlive,
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

    pub fn piece(piece_index: u32, offset: u32, block: Vec<u8>) -> PeerMessage {
        let mut payload = vec![];
        payload.extend_from_slice(&Self::u32_to_vec_be(piece_index));
        payload.extend_from_slice(&Self::u32_to_vec_be(offset));
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

pub struct PeerMessageService {
    stream: TcpStream,
    max_retries: u8,
}

impl PeerMessageService {
    pub fn connect_to_peer(peer: &Peer) -> Result<Self, PeerConnectionError> {
        trace!("Connecting to peer at IP: {}", peer.ip);
        let stream = TcpStream::connect(format!("{}:{}", peer.ip, peer.port))
            .map_err(|e| PeerConnectionError::InitialConnectionError(e.to_string()))?;
        stream
            .set_write_timeout(Some(Duration::new(MESSAGE_TIMEOUT, 0)))
            .map_err(|e| PeerConnectionError::InitialConnectionError(e.to_string()))?;
        stream
            .set_read_timeout(Some(Duration::new(MESSAGE_TIMEOUT, 0)))
            .map_err(|e| PeerConnectionError::InitialConnectionError(e.to_string()))?;
        Ok(Self {
            stream,
            max_retries: MAX_RETRIES,
        })
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

    fn try_read_exact(&mut self, buf: &mut [u8]) -> BoxedResult<()> {
        self.stream.read_exact(buf)?;
        Ok(())
    }

    fn try_write_all(&mut self, buf: &[u8]) -> BoxedResult<()> {
        self.stream.write_all(buf)?;
        Ok(())
    }

    fn write_all(&mut self, buf: &[u8]) -> BoxedResult<()> {
        let mut retries = 0;
        loop {
            match self.try_write_all(buf) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if retries >= self.max_retries {
                        return Err(e);
                    }
                    retries += 1;
                }
            }
            trace!("Attempt of sending message: {}", retries);
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> BoxedResult<()> {
        let mut retries = 0;
        loop {
            match self.try_read_exact(buf) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if retries >= self.max_retries {
                        return Err(e);
                    }
                    retries += 1;
                }
            }
            trace!("Attempt of reading message: {}", retries);
        }
    }
}

impl IPeerMessageService for PeerMessageService {
    fn wait_for_message(&mut self) -> Result<PeerMessage, IPeerMessageServiceError> {
        let mut message_length = [0u8; MESSAGE_LENGTH_SIZE];
        self.read_exact(&mut message_length).map_err(|_| {
            IPeerMessageServiceError::ReceivingMessageError(
                "Couldn't read message from other peer".to_string(),
            )
        })?;
        let message_length = u32::from_be_bytes(message_length);

        if is_keep_alive_message(message_length) {
            return self.wait_for_message();
        }

        let mut message_id = [0u8; MESSAGE_ID_SIZE];
        self.read_exact(&mut message_id).map_err(|_| {
            IPeerMessageServiceError::ReceivingMessageError(
                "Couldn't read from other peer".to_string(),
            )
        })?;

        let mut payload: Vec<u8> = vec![0; (message_length - 1) as usize];
        self.read_exact(&mut payload).map_err(|_| {
            IPeerMessageServiceError::ReceivingMessageError(
                "Couldn't read from other peer".to_string(),
            )
        })?;

        let msg = PeerMessage {
            id: PeerMessageId::from_u8(message_id[0])
                .map_err(|_| IPeerMessageServiceError::InvalidMessageId)?,
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
    ) -> Result<(), IPeerMessageServiceError> {
        let handshake_message = self.create_handshake_message(info_hash, peer_id);
        self.write_all(&handshake_message).map_err(|_| {
            IPeerMessageServiceError::SendingMessageError(
                "Couldn't send handshake message to other peer".to_string(),
            )
        })?;
        let mut handshake_response = [0u8; HANDSHAKE_LENGTH];
        self.read_exact(&mut handshake_response).map_err(|_| {
            IPeerMessageServiceError::ReceivingMessageError(
                "Couldn't read handshake from other peer".into(),
            )
        })?;
        debug!("handshake successful");
        Ok(())
    }

    fn send_message(&mut self, message: &PeerMessage) -> Result<(), IPeerMessageServiceError> {
        let mut bytes = Vec::with_capacity((message.length + 4) as usize);
        bytes.extend_from_slice(&message.length.to_be_bytes());
        bytes.extend_from_slice(&(message.id as u8).to_be_bytes());
        bytes.extend_from_slice(&message.payload);
        self.write_all(&bytes).map_err(|_| {
            IPeerMessageServiceError::SendingMessageError(
                "Couldn't send message to other peer".to_string(),
            )
        })?;
        debug!("message sent: {:?}", message.id);
        Ok(())
    }
}

pub struct PeerMessageServiceMock {
    pub counter: u32,
    pub file: Vec<u8>,
    pub block_size: u32,
}

impl IPeerMessageService for PeerMessageServiceMock {
    fn wait_for_message(&mut self) -> Result<PeerMessage, IPeerMessageServiceError> {
        let msg = PeerMessage::piece(
            0,
            self.counter * self.block_size,
            self.file[(self.counter * self.block_size) as usize
                ..(self.block_size + self.counter * self.block_size) as usize]
                .to_vec(),
        );
        self.counter += 1;
        Ok(msg)
    }

    fn handshake(
        &mut self,
        _info_hash: &[u8],
        _peer_id: &[u8],
    ) -> Result<(), IPeerMessageServiceError> {
        Ok(())
    }

    fn send_message(&mut self, _message: &PeerMessage) -> Result<(), IPeerMessageServiceError> {
        Ok(())
    }
}

pub trait IPeerMessageService {
    fn handshake(
        &mut self,
        info_hash: &[u8],
        peer_id: &[u8],
    ) -> Result<(), IPeerMessageServiceError>;
    fn wait_for_message(&mut self) -> Result<PeerMessage, IPeerMessageServiceError>;
    fn send_message(&mut self, message: &PeerMessage) -> Result<(), IPeerMessageServiceError>;
}
