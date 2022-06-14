use super::constants::*;
use super::errors::*;
use super::messages::*;
use super::utils::*;
use super::IPeerMessageServiceError;
use crate::boxed_result::BoxedResult;
use log::*;
use std::io::{Read, Write};
use std::net::SocketAddrV4;
use std::net::SocketAddrV6;
use std::net::TcpStream;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub struct Peer {
    pub ip: String,
    pub port: u16,
    pub peer_id: Vec<u8>,
}

pub struct PeerMessageService {
    stream: TcpStream,
    max_retries: u8,
}

impl PeerMessageService {
    pub fn connect_to_peer(peer: &Peer) -> Result<Self, PeerConnectionError> {
        let stream: TcpStream = if is_ipv4(&peer.ip) {
            let socket = SocketAddrV4::new(ipv4_from_str(&peer.ip)?, peer.port);
            TcpStream::connect(socket)?
        } else {
            let socket = SocketAddrV6::new(ipv6_from_str(&peer.ip)?, peer.port, 0, 0);
            TcpStream::connect(socket)?
        };

        trace!("Connecting to peer at IP: {}", peer.ip);
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
        let handshake_message = create_handshake_message(info_hash, peer_id);
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
