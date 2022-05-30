use super::errors::PeerConnectionError;
use super::types::*;
use super::utils::*;
use super::Peer;
use crate::download_manager::save_piece_in_disk;
use crate::download_manager::Piece;
use crate::logger::Logger;
use crate::metainfo::Metainfo;
use log::*;
use std::thread;

pub struct PeerConnection {
    _am_choking: bool,
    _am_interested: bool,
    peer_choking: bool,
    _peer_interested: bool,
    message_service: Box<dyn PeerMessageService>,
    metainfo: Metainfo,
    client_peer_id: Vec<u8>,
    bitfield: Bitfield,
}

impl PeerConnection {
    pub fn new(
        _peer: &Peer,
        client_peer_id: &[u8],
        metainfo: &Metainfo,
        message_service: Box<dyn PeerMessageService>,
    ) -> Self {
        Self {
            _am_choking: true,
            _am_interested: true,
            peer_choking: true,
            _peer_interested: false,
            client_peer_id: client_peer_id.to_vec(),
            metainfo: metainfo.clone(),
            message_service,
            bitfield: Bitfield::new(),
        }
    }

    fn wait_for_message(&mut self) -> Result<PeerMessage, Box<dyn std::error::Error>> {
        let message = self.message_service.wait_for_message()?;
        match message.id {
            PeerMessageId::Unchoke => {
                self.peer_choking = false;
            }
            PeerMessageId::Bitfield => {
                self.bitfield.set_bitfield(&message.payload);
            }
            PeerMessageId::Piece => {}
            _ => {
                return Err("unhandled message".into());
            }
        }
        Ok(message)
    }

    fn wait_until_ready(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            self.wait_for_message()?;
            if !self.peer_choking && self.bitfield.non_empty() {
                break;
            }
        }
        Ok(())
    }

    // TODO: handlear errores
    // Requests a block of data of some piece (index refers to the index of the piece).
    // Data starts from the offset within the piece, and its size is the length requested.
    // Once a block is recieved, it is checked if it is valid, and if it is, it is returned.
    fn request_block(
        &mut self,
        index: u32,
        begin: u32,
        lenght: u32,
    ) -> Result<Vec<u8>, PeerConnectionError> {
        self.message_service
            .send_message(&PeerMessage::request(index, begin, lenght))?;
        loop {
            let message = self.wait_for_message()?;
            if message.id == PeerMessageId::Piece {
                if valid_block(&message.payload, index, begin) {
                    let block = message.payload[8..].to_vec();
                    debug!("block received");
                    break Ok(block);
                } else {
                    break Err(PeerConnectionError("Invalid block recieved".to_string()));
                }
            }
        }
    }

    // Requests a specific piece from the peer.
    // It does it sequentially, by requesting blocks of data, until the whole piece is recieved.
    // Once it is complete, we verify its sha1 hash, and return the piece if it is valid.
    fn request_piece(
        &mut self,
        piece_index: u32,
        block_size: u32,
    ) -> Result<Vec<u8>, PeerConnectionError> {
        let mut counter = 0;
        let mut piece: Vec<u8> = vec![];
        debug!("requesting piece: {}", piece_index);
        loop {
            let block: Vec<u8> = self.request_block(piece_index, counter, block_size)?;
            piece.extend(block);
            counter += block_size;
            if counter >= self.metainfo.info.piece_length {
                if valid_piece(&piece, piece_index, &self.metainfo) {
                    debug!("recieved full piece, piece index: {}", piece_index);
                    break Ok(piece);
                } else {
                    break Err(PeerConnectionError("Invalid piece recieved".to_string()));
                }
            }
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (logger, mut worker) = Logger::new("./logs").unwrap();
        let join_handle = thread::spawn(move || {
            worker.listen().unwrap();
        });

        self.message_service
            .handshake(&self.metainfo.info_hash, &self.client_peer_id)?;
        self.message_service.send_message(&PeerMessage::unchoke())?;
        self.message_service
            .send_message(&PeerMessage::interested())?;
        self.wait_until_ready()?;
        const BLOCK_SIZE: u32 = 16 * u32::pow(2, 10);
        let piece_data: Vec<u8> = self
            .request_piece(0, BLOCK_SIZE)
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)?;

        let piece = Piece {
            piece_number: 0,
            data: piece_data,
        };

        debug!("saving downloaded piece {} in disk", piece.piece_number);
        save_piece_in_disk(&piece, "./downloads").unwrap();
        debug!("logging downloaded piece");
        logger.log_piece(0).unwrap();

        logger.stop_logging().unwrap();
        join_handle.join().unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metainfo::Info;
    use crate::metainfo::Metainfo;

    #[test]
    fn gets_real_piece() {
        let file = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut pieces = sha1_of(&file[0..8].to_vec());
        pieces.extend(sha1_of(&file[8..16].to_vec()));

        let metainfo_mock = Metainfo {
            announce: "".to_string(),
            info: Info {
                piece_length: 8,
                pieces: pieces,
                length: 16,
                name: "".to_string(),
            },
            info_hash: vec![],
        };

        let peer_mock = Peer {
            ip: "".to_string(),
            port: 0,
            peer_id: vec![],
        };
        const BLOCK_SIZE: u32 = 2;
        let peer_message_stream_mock = PeerMessageStreamMock {
            counter: 0,
            file: file.clone(),
            block_size: BLOCK_SIZE,
        };
        let mut peer_connection = PeerConnection::new(
            &peer_mock,
            &vec![1, 2, 3, 4],
            &metainfo_mock,
            Box::new(peer_message_stream_mock),
        );

        let piece = peer_connection.request_piece(0, BLOCK_SIZE);
        assert_eq!(file[0..8], piece.unwrap());
    }

    #[test]
    fn gets_invalid_block() {
        let file = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut pieces = sha1_of(&file[0..8].to_vec());
        pieces.extend(sha1_of(&file[8..16].to_vec()));

        let metainfo_mock = Metainfo {
            announce: "".to_string(),
            info: Info {
                piece_length: 8,
                pieces: pieces,
                length: 16,
                name: "".to_string(),
            },
            info_hash: vec![],
        };

        let peer_mock = Peer {
            ip: "".to_string(),
            port: 0,
            peer_id: vec![],
        };
        const BLOCK_SIZE: u32 = 2;
        let peer_message_stream_mock = PeerMessageStreamMock {
            counter: 0,
            file: file.clone(),
            block_size: BLOCK_SIZE,
        };
        let mut peer_connection = PeerConnection::new(
            &peer_mock,
            &vec![1, 2, 3, 4],
            &metainfo_mock,
            Box::new(peer_message_stream_mock),
        );

        assert!(matches!(
            peer_connection.request_piece(1, BLOCK_SIZE),
            Err(PeerConnectionError(_))
        ));
    }
}
