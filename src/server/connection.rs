use super::constants::*;
use super::errors::ServerError;
use super::utils::*;
use crate::metainfo::Metainfo;
use crate::peer::IServerPeerMessageService;
use crate::peer::PeerMessage;
use crate::peer::PeerMessageId;
use log::*;

#[allow(dead_code)]
pub struct ServerConnection {
    message_service: Box<dyn IServerPeerMessageService>,
    metainfo: Metainfo,
    client_peer_id: Vec<u8>,
}

#[allow(dead_code)]
pub struct RequestMessage {
    pub index: usize,
    pub begin: usize,
    pub length: usize,
}

impl ServerConnection {
    pub fn new(
        client_peer_id: Vec<u8>,
        metainfo: Metainfo,
        message_service: Box<dyn IServerPeerMessageService>,
    ) -> Self {
        Self {
            client_peer_id: client_peer_id.to_vec(),
            metainfo,
            message_service,
        }
    }

    pub fn run(&mut self) -> Result<(), ServerError> {
        self.message_service
            .handshake(&self.metainfo.info_hash, &self.client_peer_id)
            .unwrap();

        loop {
            let message: PeerMessage = match self.message_service.wait_for_message() {
                Ok(message) => message,
                Err(_) => {
                    debug!("Server connection was closed by client or timeout ocurred");
                    break;
                }
            };

            trace!("Server: received message: {:?}", message);
            match message.id {
                PeerMessageId::Request => self.handle_request(message)?,
                PeerMessageId::KeepAlive => continue,
                PeerMessageId::Interested => continue,
                PeerMessageId::Unchoke => continue,
                PeerMessageId::Bitfield => continue,
                PeerMessageId::Have => continue,
                PeerMessageId::Piece => continue,
                PeerMessageId::Port => continue,
                PeerMessageId::Choke => break,
                PeerMessageId::Cancel => break,
                PeerMessageId::NotInterested => break,
            }
        }

        Ok(())
    }

    // In the future, we migth want to cache the piece when the first request message is received for a piece
    fn handle_request(&mut self, message: PeerMessage) -> Result<(), ServerError> {
        let request: RequestMessage = request_from_payload(message.payload)?;
        trace!("Received request message for piece: {}", request.index);
        if !client_has_piece(request.index, PIECES_DIR) {
            trace!("Received piece request for piece that client does not have, ignoring for the moment");
            return Ok(());
        }

        let piece_path = format!("{}/{}", PIECES_DIR, request.index);
        let piece_data: Vec<u8> = read_piece(&piece_path)?;
        let block: Vec<u8> = get_block_from_piece(piece_data, request.begin, request.length);

        let response_message: PeerMessage = PeerMessage::piece(request.index, request.begin, block);
        self.message_service.send_message(&response_message)?;
        Ok(())
    }
}
