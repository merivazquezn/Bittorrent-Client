use super::errors::IPeerMessageServiceError;
use super::errors::PeerConnectionError;
use super::service::*;
use super::types::*;
use super::utils::*;
use super::Peer;
use crate::constants::*;
use crate::metainfo::Metainfo;
use crate::ui::UIMessageSender;
use log::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct PeerConnection {
    pub _am_choking: bool,
    pub _am_interested: bool,
    pub peer_choking: bool,
    pub _peer_interested: bool,
    pub message_service: Box<dyn IClientPeerMessageService + Send>,
    pub metainfo: Metainfo,
    pub client_peer_id: Vec<u8>,
    pub bitfield: Bitfield,
    pub peer_id: Vec<u8>,
    pub peer: Peer,
    pub last_download_rate_update: std::time::Instant,
    pub last_downloaded_pieces: Arc<AtomicUsize>,
    pub ui_message_sender: UIMessageSender,
}

impl PeerConnection {
    pub fn new(
        peer: Peer,
        client_peer_id: &[u8],
        metainfo: &Metainfo,
        message_service: Box<dyn IClientPeerMessageService + Send>,
        ui_message_sender: UIMessageSender,
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
            peer_id: peer.peer_id.clone(),
            last_downloaded_pieces: Arc::new(AtomicUsize::new(0)),
            last_download_rate_update: std::time::Instant::now(),
            ui_message_sender,
            peer,
        }
    }
    pub fn get_peer_id(&self) -> Vec<u8> {
        self.peer_id.clone()
    }
    pub fn get_peer_ip(&self) -> String {
        self.peer.ip.clone()
    }

    pub fn get_bitfield(&self) -> Bitfield {
        self.bitfield.clone()
    }

    fn wait_for_message(&mut self) -> Result<PeerMessage, IPeerMessageServiceError> {
        let message = self.message_service.wait_for_message()?;
        match message.id {
            PeerMessageId::Unchoke => {
                self.peer_choking = false;
            }
            PeerMessageId::Choke => {
                self.peer_choking = true;
            }
            PeerMessageId::Interested => {
                self._peer_interested = true;
            }
            PeerMessageId::NotInterested => {
                self._peer_interested = false;
            }
            PeerMessageId::Bitfield => {
                self.bitfield.set_bitfield(&message.payload);
            }
            PeerMessageId::Have => {}
            PeerMessageId::Piece => {}
            _ => {
                return Err(IPeerMessageServiceError::UnhandledMessage);
            }
        }
        Ok(message)
    }

    fn wait_until_ready(&mut self) -> Result<(), IPeerMessageServiceError> {
        loop {
            self.wait_for_message()?;
            self.ui_message_sender.update_peer_state(
                self.peer_id.clone(),
                PeerConnectionState {
                    client: (PeerState {
                        chocked: self.peer_choking,
                        interested: self._am_interested,
                    }),
                    peer: (PeerState {
                        chocked: self._am_choking,
                        interested: self._peer_interested,
                    }),
                },
            );

            if !self.peer_choking && self.bitfield.non_empty() {
                break;
            }
        }
        Ok(())
    }

    // Requests a block of data of some piece (index refers to the index of the piece).
    // Data starts from the offset within the piece, and its size is the length requested.
    // Once a block is recieved, it is checked if it is valid, and if it is, it is returned.
    fn request_block(
        &mut self,
        index: u32,
        begin: u32,
        lenght: u32,
        _ui_message_sender: UIMessageSender,
    ) -> Result<Vec<u8>, PeerConnectionError> {
        let _block_count = self.metainfo.info.piece_length / BLOCK_SIZE;

        // calculate duration between sending the message and moving on to next instruction
        let msg = PeerMessage::request(index, begin, lenght);
        self.message_service.send_message(&msg)?;

        loop {
            let message = self.wait_for_message().map_err(|_| {
                PeerConnectionError::PieceRequestingError("Failed while waiting for message".into())
            })?;

            if message.id == PeerMessageId::Piece {
                if valid_block(&message.payload, index, begin) {
                    let block = message.payload[8..].to_vec();
                    break Ok(block);
                } else {
                    break Err(PeerConnectionError::PieceRequestingError(
                        "Invalid block received".to_string(),
                    ));
                }
            }
        }
    }

    // Requests a specific piece from the peer.
    // It does it sequentially, by requesting blocks of data, until the whole piece is recieved.
    // Returns the piece unchecked
    pub fn request_piece(
        &mut self,
        piece_index: u32,
        block_size: u32,
        ui_message_sender: UIMessageSender,
    ) -> Result<Vec<u8>, PeerConnectionError> {
        let mut counter = 0;
        let mut piece: Vec<u8> = vec![];
        debug!("requesting piece: {}", piece_index);
        while counter < self.metainfo.info.piece_length {
            let ui_sender_clone = ui_message_sender.clone();
            let block: Vec<u8> =
                self.request_block(piece_index, counter, block_size, ui_sender_clone)?;
            piece.extend(block);
            counter += block_size;
        }

        self.last_downloaded_pieces.fetch_add(1, Ordering::Relaxed);

        if self.last_downloaded_pieces.load(Ordering::Relaxed) == 1 {
            let time = self.last_download_rate_update.elapsed().as_secs_f32();
            self.ui_message_sender.send_download_rate(
                2f32 * self.metainfo.info.piece_length as f32 / time,
                &self.get_peer_id(),
            );
            self.last_download_rate_update = std::time::Instant::now();
            self.last_downloaded_pieces.store(0, Ordering::Relaxed);
        }
        debug!(
            "recieved piece (not validated yet), piece index: {}",
            piece_index
        );
        Ok(piece)
    }

    //Executes all steps needed to start an active connection with Peer
    pub fn open_connection(&mut self) -> Result<(), PeerConnectionError> {
        self.message_service
            .handshake(&self.metainfo.info_hash, &self.client_peer_id)
            .map_err(|_| {
                IPeerMessageServiceError::PeerHandshakeError("Handshake error".to_string())
            })?;

        self.message_service
            .send_message(&PeerMessage::unchoke())
            .map_err(|_| {
                IPeerMessageServiceError::SendingMessageError(
                    "Error trying to send unchoke message".to_string(),
                )
            })?;

        self.message_service
            .send_message(&PeerMessage::interested())
            .map_err(|_| {
                IPeerMessageServiceError::SendingMessageError(
                    "Error trying to send interested message".to_string(),
                )
            })?;
        self.wait_until_ready()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metainfo::Info;
    use crate::metainfo::Metainfo;
    use sha1::{Digest, Sha1};

    fn get_pieces_hash_from_bytes(file: &Vec<u8>) -> Vec<Vec<u8>> {
        let mut pieces = Vec::new();
        for chunk in file.chunks(8 as usize) {
            let mut hasher = Sha1::new();
            hasher.update(chunk);
            pieces.push(hasher.finalize()[..].to_vec());
        }
        pieces
    }
    #[test]
    fn gets_real_piece() {
        let file = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut pieces: Vec<Vec<u8>> = Vec::new();
        pieces.push(sha1_of(&file[0..8].to_vec()));
        pieces.push(sha1_of(&file[8..16].to_vec()));

        let metainfo_mock = Metainfo {
            announce: "".to_string(),
            info: Info {
                piece_length: 8,
                pieces: get_pieces_hash_from_bytes(&file),
                length: file.len() as u64,
                name: "".to_string(),
                files: None,
            },
            info_hash: vec![],
        };

        let peer_mock = Peer {
            ip: "".to_string(),
            port: 0,
            peer_id: vec![],
            peer_message_service_provider: mock_peer_message_service_provider,
        };
        let peer_message_stream_mock = PeerMessageServiceMock {
            counter: 0,
            file: file.clone(),
            block_size: 2 as u32,
        };
        let mut peer_connection = PeerConnection::new(
            peer_mock,
            &vec![1, 2, 3, 4],
            &metainfo_mock,
            Box::new(peer_message_stream_mock),
            UIMessageSender::no_ui(),
        );

        // measure time spent requesting a piece
        let piece = peer_connection
            .request_piece(0, 2 as u32, UIMessageSender::no_ui())
            .unwrap();
        assert_eq!(file[0..8], piece);
    }

    #[test]
    fn gets_invalid_block() {
        let file = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut pieces: Vec<Vec<u8>> = Vec::new();
        pieces.push(sha1_of(&file[0..8].to_vec()));
        pieces.push(sha1_of(&file[8..16].to_vec()));

        let metainfo_mock = Metainfo {
            announce: "".to_string(),
            info: Info {
                piece_length: 8,
                pieces: pieces,
                length: 16,
                name: "".to_string(),
                files: None,
            },
            info_hash: vec![],
        };

        let peer_mock = Peer {
            ip: "".to_string(),
            port: 0,
            peer_id: vec![],
            peer_message_service_provider: mock_peer_message_service_provider,
        };
        let peer_message_stream_mock = PeerMessageServiceMock {
            counter: 0,
            file: file.clone(),
            block_size: 2 as u32,
        };
        let mut peer_connection = PeerConnection::new(
            peer_mock,
            &vec![1, 2, 3, 4],
            &metainfo_mock,
            Box::new(peer_message_stream_mock),
            UIMessageSender::no_ui(),
        );

        assert!(matches!(
            peer_connection.request_piece(1, BLOCK_SIZE, UIMessageSender::no_ui()),
            Err(PeerConnectionError::PieceRequestingError(_))
        ));
    }
}
