use super::errors::ServerError;
use super::logger::ServerLogger;
use super::utils::*;
use crate::metainfo::Metainfo;
use crate::peer::IServerPeerMessageService;
use crate::peer::PeerMessage;
use crate::peer::PeerMessageId;
use log::*;

/// Struct that handles the server's acceptor thread.
/// It is spawned each time a connection is accepted.
/// It handles the connection's messages and answers them accordingly.
pub struct ServerConnection {
    message_service: Box<dyn IServerPeerMessageService>,
    metainfo: Metainfo,
    client_peer_id: Vec<u8>,
}

/// Struct representing the content of a request message
pub struct RequestMessage {
    /// The piece index of the requested piece
    pub index: usize,
    /// The offset of the requested block
    pub begin: usize,
    /// The length of the requested block
    pub length: usize,
}

impl ServerConnection {
    /// Creates a new server connection.
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

    /// Runs a server connection which will hear messages from other peers and answer accordingly
    /// The connectcion starts listening inmediatly after calling this method
    ///
    /// The connection has a timeout of 120 seconds, so that it can be automatically closed if no message is received after that interval
    /// Messagges requesting a block are answered, while Choke, Cancel and NotInterested messges close the connection.
    /// Every other message is ignored.
    ///
    ///  If an invalid request is received, the connection is terminated
    ///
    /// # Arguments
    /// * `logger` - The server logger to use for logging.
    /// * `pieces_dir` - The directory where the pieces are stored.
    ///
    /// # Return value
    /// ## On succes
    /// A `Result` with the `Ok` value being `()`
    ///
    /// ## On error
    /// A `Result` with the `Err` value being a `ServerError`, indicating the underlying cause of the failure
    ///
    pub fn run(&mut self, logger: ServerLogger, pieces_dir: &str) -> Result<(), ServerError> {
        info!("before init messages");
        self.send_init_messages(pieces_dir)?;
        info!("after init messages, about to wait for message from client");

        loop {
            let message: PeerMessage = match self.message_service.wait_for_message() {
                Ok(message) => {
                    info!("message from client got: {:?}", message);
                    let cloned_message = message.clone();
                    let _ = logger.received_message(cloned_message);
                    message
                }
                Err(_) => {
                    debug!("Server connection was closed by client or timeout ocurred");
                    break;
                }
            };

            let cloned_logger = logger.clone();
            match message.id {
                PeerMessageId::Request => {
                    self.handle_request(message, cloned_logger, pieces_dir)?;
                    continue;
                }
                PeerMessageId::KeepAlive => continue,
                PeerMessageId::Interested => continue,
                PeerMessageId::Unchoke => continue,
                PeerMessageId::Bitfield => continue,
                PeerMessageId::Have => continue,
                PeerMessageId::Piece => continue,
                PeerMessageId::Port => continue,
                PeerMessageId::Cancel => break,
                PeerMessageId::Choke => break,
                PeerMessageId::NotInterested => break,
            };
        }

        Ok(())
    }

    fn send_init_messages(&mut self, download_path: &str) -> Result<(), ServerError> {
        self.message_service
            .handshake(&self.metainfo.info_hash, &self.client_peer_id)?;

        self.message_service.send_message(&PeerMessage::unchoke())?;

        let piece_vector: Vec<bool> =
            get_pieces_vector(self.metainfo.info.pieces.len(), download_path);
        let bitfield_message: PeerMessage = PeerMessage::bitfield(piece_vector);

        self.message_service.send_message(&bitfield_message)?;
        Ok(())
    }

    fn handle_request(
        &mut self,
        message: PeerMessage,
        logger: ServerLogger,
        pieces_dir: &str,
    ) -> Result<(), ServerError> {
        let request: RequestMessage = request_from_payload(message.payload)?;
        if !client_has_piece(request.index, pieces_dir) {
            let _ = logger.client_doesnt_have_piece(request.index);
            return Ok(());
        }

        let piece_path = format!("{}/{}", pieces_dir, request.index);
        let piece_data: Vec<u8> = read_piece(&piece_path)?;
        let block: Vec<u8> = get_block_from_piece(piece_data, request.begin, request.length)?;
        let block_number: usize = get_block_index(request.begin, request.length);
        // sleep for having a feeling of internet download
        std::thread::sleep(std::time::Duration::from_millis(3000));
        let response_message = PeerMessage::piece(request.index, request.begin, block);
        match self.message_service.send_message(&response_message) {
            Ok(()) => {
                let _ = logger.block_sent_succesfully(request.index, block_number);
            }
            Err(_) => {
                let _ = logger.failed_sending_block(request.index, block_number);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::metainfo::Info;
    use crate::peer::ServerMessageServiceMock;
    use sha1::{Digest, Sha1};

    pub fn sha1_of(vec: &[u8]) -> Vec<u8> {
        let mut hasher = Sha1::new();
        hasher.update(vec);
        hasher.finalize().to_vec()
    }

    fn get_fake_peer_id() -> Vec<u8> {
        let mut peer_id: Vec<u8> = Vec::new();
        for i in 0..20 {
            peer_id.push(i as u8);
        }
        peer_id
    }

    fn get_fake_metainfo() -> Metainfo {
        let file = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0];
        let mut pieces: Vec<Vec<u8>> = Vec::new();
        pieces.push(sha1_of(&file[0..8].to_vec()));
        pieces.push(sha1_of(&file[8..16].to_vec()));
        Metainfo {
            announce: "".to_string(),
            info: Info {
                piece_length: 8,
                pieces: pieces,
                length: 16,
                name: "".to_string(),
                files: None,
            },
            info_hash: vec![],
        }
    }

    fn get_mock_message_service() -> Box<dyn IServerPeerMessageService> {
        Box::new(ServerMessageServiceMock { times_called: 0 })
    }

    fn write_piece(piece: &[u8], piece_index: usize, pieces_dir: &str) -> Result<(), ServerError> {
        use std::fs::File;
        use std::io::Write;
        let piece_path = format!("{}/{}", pieces_dir, piece_index);
        let mut file = File::create(piece_path)?;
        file.write_all(piece)?;
        file.flush().unwrap();
        Ok(())
    }

    fn read_lines_from_file(file_path: &str) -> Vec<String> {
        use std::fs::File;
        use std::io::Read;
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
        lines
    }

    #[test]
    fn peer_asks_for_piece_server_doesnt_have_it_log_client_has_no_piece() {
        // arrange
        let peer_id = get_fake_peer_id();
        let metainfo = get_fake_metainfo();

        let message_service = get_mock_message_service();
        let mut connection = ServerConnection::new(peer_id, metainfo, message_service);

        let pieces_dir: &str = "./src/server/tests/test_2/pieces";
        let logs_dir: &str = "./src/server/tests/test_2/logs";

        let (logger, handle) = ServerLogger::new(logs_dir).unwrap();
        let logger_clone = logger.clone();

        // act
        connection.run(logger_clone, pieces_dir).unwrap();
        logger.stop();
        handle.join().unwrap();

        let lines: Vec<String> = read_lines_from_file(&format!("{}/server_log.txt", logs_dir));
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1], "Client doesn't have piece 0");
    }

    #[test]
    fn peer_asks_for_piece_client_has_it_succesfully_logged() {
        // arrange
        let peer_id = get_fake_peer_id();
        let metainfo = get_fake_metainfo();

        let message_service = get_mock_message_service();
        let mut connection = ServerConnection::new(peer_id, metainfo, message_service);

        let pieces_dir: &str = "./src/server/tests/test_1/pieces";
        let logs_dir: &str = "./src/server/tests/test_1/logs";
        write_piece(vec![0, 0, 0, 0, 0, 0, 0, 1].as_slice(), 0, pieces_dir).unwrap();

        let (logger, handle) = ServerLogger::new(logs_dir).unwrap();
        let logger_clone = logger.clone();

        // act
        connection.run(logger_clone, pieces_dir).unwrap();
        logger.stop();
        handle.join().unwrap();

        // assert
        let lines: Vec<String> = read_lines_from_file(&format!("{}/server_log.txt", logs_dir));
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[1], "Block 0 of piece 0 succesfully sent");
    }

    #[test]
    fn server_sends_unchocke_and_bitfield_when_connection_starts() {
        // arrange
        let peer_id = get_fake_peer_id();
        let metainfo = get_fake_metainfo();

        use crate::peer::ServerMessageBitfieldMock;
        let mut connection =
            ServerConnection::new(peer_id, metainfo, Box::new(ServerMessageBitfieldMock));

        let pieces_dir: &str = "./src/server/tests/test_3/pieces";
        let logs_dir: &str = "./src/server/tests/test_3/logs";

        let (logger, handle) = ServerLogger::new(logs_dir).unwrap();
        let logger_clone = logger.clone();

        // act
        connection.run(logger_clone, pieces_dir).unwrap();
        logger.stop();
        handle.join().unwrap();

        // assert
        let lines: Vec<String> = read_lines_from_file(&format!(
            "./src/server/tests/test_3/initialize_connection.txt"
        ));

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "handshake");
        assert_eq!(lines[1], "1");
        assert_eq!(lines[2], "5")
    }
}
