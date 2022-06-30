use bittorrent_rustico::constants::*;
use bittorrent_rustico::peer::*;
use std::vec::Vec;

struct PeerMessageServiceMockExtended {
    pub counter: u32,
    pub file: Vec<u8>,
    pub block_size: u32,
    pub unchoke_sent: bool,
    pub bitfield_sent: bool,
    pub bitfield: Vec<bool>,
    pub piece_array: Vec<usize>,
}

impl IPeerMessageService for PeerMessageServiceMockExtended {
    fn wait_for_message(&mut self) -> Result<PeerMessage, IPeerMessageServiceError> {
        if !self.unchoke_sent {
            self.unchoke_sent = true;
            return Ok(PeerMessage::unchoke());
        }

        if !self.bitfield_sent {
            self.bitfield_sent = true;
            return Ok(PeerMessage::bitfield(self.bitfield.clone()));
        }
        let msg = PeerMessage::piece(
            self.piece_array[self.counter as usize],
            (self.counter * self.block_size) as usize,
            self.file[(self.counter * self.block_size) as usize
                ..(self.block_size + self.counter * self.block_size) as usize]
                .to_vec(),
        );
        self.counter += 1;
        Ok(msg)
    }

    fn send_message(&mut self, _message: &PeerMessage) -> Result<(), IPeerMessageServiceError> {
        Ok(())
    }
}

impl IClientPeerMessageService for PeerMessageServiceMockExtended {
    fn handshake(
        &mut self,
        _info_hash: &[u8],
        _peer_id: &[u8],
    ) -> Result<(), IPeerMessageServiceError> {
        Ok(())
    }
}

fn create_file_1() -> Vec<u8> {
    let mut vec = Vec::new();
    for _ in 0..BLOCK_SIZE {
        vec.push(1);
    }
    vec
}

fn create_file_2() -> Vec<u8> {
    let mut vec = Vec::new();
    for _ in 0..BLOCK_SIZE {
        vec.push(2);
    }
    vec
}

fn create_file_3() -> Vec<u8> {
    let mut vec = Vec::new();
    for _ in 0..BLOCK_SIZE {
        vec.push(3);
    }
    vec
}
pub fn mock_peer_message_service_1(
    _ip: String,
    _port: u16,
) -> Result<Box<dyn IClientPeerMessageService + Send>, PeerConnectionError> {
    Ok(Box::new(PeerMessageServiceMockExtended {
        counter: 0,
        file: create_file_1(),
        block_size: BLOCK_SIZE,
        unchoke_sent: false,
        bitfield_sent: false,
        bitfield: vec![true, false, false],
        piece_array: vec![0],
    }))
}

pub fn mock_peer_message_service_2(
    _ip: String,
    _port: u16,
) -> Result<Box<dyn IClientPeerMessageService + Send>, PeerConnectionError> {
    Ok(Box::new(PeerMessageServiceMockExtended {
        counter: 0,
        file: create_file_2(),
        block_size: BLOCK_SIZE,
        unchoke_sent: false,
        bitfield_sent: false,
        bitfield: vec![false, true, false],
        piece_array: vec![1],
    }))
}

pub fn mock_peer_message_service_3(
    _ip: String,
    _port: u16,
) -> Result<Box<dyn IClientPeerMessageService + Send>, PeerConnectionError> {
    Ok(Box::new(PeerMessageServiceMockExtended {
        counter: 0,
        file: create_file_3(),
        block_size: BLOCK_SIZE,
        unchoke_sent: false,
        bitfield_sent: false,
        bitfield: vec![false, false, true],
        piece_array: vec![2],
    }))
}
