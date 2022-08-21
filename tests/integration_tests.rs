use bittorrent_rustico::client::*;
use bittorrent_rustico::config::*;
use bittorrent_rustico::constants::*;
use bittorrent_rustico::metainfo::*;
use bittorrent_rustico::peer::*;
use bittorrent_rustico::ui::*;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;
mod mock_service_creation;
use bittorrent_rustico::metainfo::{self, Metainfo};
use bittorrent_rustico::server::Server;
use bittorrent_rustico::tracker::MockTrackerService;
use bittorrent_rustico::tracker::TrackerService;
use mock_service_creation::*;
use rand::Rng;
use std::net::TcpStream;

fn get_mock_tracker_responses() -> Vec<Vec<Peer>> {
    let peer_0 = Peer {
        ip: String::from("0.0.0.0"),
        port: 0,
        peer_id: vec![0],
        peer_message_service_provider: mock_peer_message_service_0,
    };
    let peer_1 = Peer {
        ip: String::from("1.1.1.1"),
        port: 0,
        peer_id: vec![1],
        peer_message_service_provider: mock_peer_message_service_1,
    };
    let peer_2 = Peer {
        ip: String::from("2.2.2.2"),
        port: 0,
        peer_id: vec![2],
        peer_message_service_provider: mock_peer_message_service_2,
    };
    let _faulty_peer = Peer {
        ip: String::from("9.9.9.9"),
        port: 0,
        peer_id: vec![99],
        peer_message_service_provider: mock_faulty_peer_message_service,
    };

    vec![
        vec![peer_1, peer_2, peer_0],
        // vec![peer_1.clone(), faulty_peer.clone()],
        // vec![faulty_peer.clone(), peer_2, peer_0],
    ]
}

fn get_pieces_hash_from_bytes(file: &Vec<u8>) -> Vec<Vec<u8>> {
    let mut pieces = Vec::new();
    for chunk in file.chunks(BLOCK_SIZE as usize) {
        let mut hasher = Sha1::new();
        hasher.update(chunk);
        pieces.push(hasher.finalize()[..].to_vec());
    }
    pieces
}

// check if a directory exists
fn dir_exists(path: &str) -> bool {
    std::fs::metadata(path).is_ok()
}

fn setup() {
    pretty_env_logger::init();
    if dir_exists("./tests/downloads/test_server/pieces") {
        std::fs::remove_dir_all("./tests/downloads/test_server/pieces").unwrap();
    }

    let downloads_dir_path = "./tests/downloads/test_server/pieces";
    std::fs::create_dir_all(downloads_dir_path).unwrap();
}

#[test]
fn client_integration_test() {
    setup();
    let mut file = Vec::new();

    for _ in 0..BLOCK_SIZE {
        file.push(PIECE_0_BYTES);
    }
    for _ in 0..BLOCK_SIZE {
        file.push(PIECE_1_BYTES);
    }
    for _ in 0..BLOCK_SIZE {
        file.push(PIECE_2_BYTES);
    }

    let info = Info {
        piece_length: BLOCK_SIZE,
        pieces: get_pieces_hash_from_bytes(&file),
        name: String::from("linux_distribution_test.iso"),
        length: file.len() as u64,
        files: None,
    };
    let metainfo = Metainfo {
        announce: String::from("mock_url"),
        info_hash: vec![],
        info,
    };

    let tracker_responses = get_mock_tracker_responses();
    let client_info = ClientInfo {
        config: Config::from_path("tests/test_config.txt").unwrap(),
        peer_id: generate_peer_id(),
        metainfo,
    };
    let client: TorrentClient =
        TorrentClient::new(&client_info, UIMessageSender::no_ui(), vec![]).unwrap();

    let mut tracker_service = MockTrackerService {
        responses: tracker_responses.to_vec(),
        response_index: 0,
    };

    client
        .run(client_info, &mut tracker_service) // antes recibia UIMessageSender::no_ui()
        .unwrap();
    let mut entire_file: File = File::open(
        "./tests/downloads/linux_distribution_test.iso/target/linux_distribution_test.iso",
    )
    .unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let _ = entire_file.read_to_end(&mut buf).unwrap();
    assert_eq!(file, buf);
}

fn get_metainfo(pieces: Vec<Vec<u8>>, info_hash: Vec<u8>) -> Metainfo {
    let announce: String = "127.0.0.1".to_string();

    let info: Info = Info {
        piece_length: 8,
        pieces,
        name: "target.txt".to_string(),
        length: 24, // 3 pieces of 8 bytes each
        files: None::<Vec<metainfo::File>>,
    };

    Metainfo {
        announce,
        info,
        info_hash,
    }
}

// Returns a vector of the pieces each piece
fn init_pieces() -> (Vec<Vec<u8>>, Vec<u8>) {
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    let mut joined_pieces: Vec<u8> = Vec::new();

    std::fs::create_dir_all("./downloads/test_server/pieces").unwrap();
    let mut file_0 = File::create("./downloads/test_server/pieces/0").unwrap();
    let mut buf_0: Vec<u8> = Vec::new();
    for i in 0..8 {
        buf_0.push(i as u8);
        joined_pieces.push(i as u8);
    }
    pieces.push(buf_0.clone());

    file_0.write_all(buf_0.as_slice()).unwrap();

    let mut file_1 = File::create("./downloads/test_server/pieces/1").unwrap();
    let mut buf_1: Vec<u8> = Vec::new();
    for i in 0..8 {
        buf_1.push((8 - i) as u8);
        joined_pieces.push((8 - i) as u8)
    }
    pieces.push(buf_1.clone());
    file_1.write_all(buf_1.as_slice()).unwrap();

    let mut file_2 = File::create("./downloads/test_server/pieces/2").unwrap();
    let mut buf_2: Vec<u8> = Vec::new();
    for _ in 0..8 {
        buf_2.push(3 as u8);
        joined_pieces.push(3 as u8);
    }
    pieces.push(buf_2.clone());
    file_2.write_all(buf_2.as_slice()).unwrap();

    (pieces, sha1_of(&joined_pieces))
}

fn send_message(
    stream: &mut TcpStream,
    message: &PeerMessage,
) -> Result<(), IPeerMessageServiceError> {
    let mut bytes = Vec::with_capacity((message.length + 4) as usize);
    bytes.extend_from_slice(&message.length.to_be_bytes());
    bytes.extend_from_slice(&(message.id as u8).to_be_bytes());
    bytes.extend_from_slice(&message.payload);
    stream.write_all(&bytes).map_err(|_| {
        IPeerMessageServiceError::SendingMessageError(
            "Couldn't send message to other peer".to_string(),
        )
    })?;
    Ok(())
}

fn wait_for_message(stream: &mut TcpStream) -> Result<PeerMessage, IPeerMessageServiceError> {
    let mut message_length = [0u8; 4];
    stream.set_nonblocking(false)?;
    stream.read_exact(&mut message_length).map_err(|_| {
        IPeerMessageServiceError::ReceivingMessageError(
            "Couldn't read message from other peer".to_string(),
        )
    })?;

    let message_length = u32::from_be_bytes(message_length);

    let mut message_id = [0u8; 1];
    stream.read_exact(&mut message_id).map_err(|_| {
        IPeerMessageServiceError::ReceivingMessageError("Couldn't read from other peer".to_string())
    })?;

    let mut payload: Vec<u8> = vec![0; (message_length - 1) as usize];
    stream.read_exact(&mut payload).map_err(|_| {
        IPeerMessageServiceError::ReceivingMessageError("Couldn't read from other peer".to_string())
    })?;

    let msg = PeerMessage {
        id: PeerMessageId::from_u8(message_id[0])
            .map_err(|_| IPeerMessageServiceError::InvalidMessageId)?,
        length: message_length,
        payload,
    };

    Ok(msg)
}

fn init_connection(stream: &mut TcpStream, meta: &Metainfo, peer_id: &Vec<u8>) -> bool {
    let handshake_message: Vec<u8> = create_handshake_message(&meta.info_hash, &peer_id);
    stream.write_all(&handshake_message).unwrap();
    let mut handshake_response = [0u8; 68];
    stream.read_exact(&mut handshake_response).unwrap();

    let unchocke_message: PeerMessage = wait_for_message(stream).unwrap();
    if unchocke_message.id != PeerMessageId::Unchoke {
        return false;
    }

    let bitfield_message: PeerMessage = wait_for_message(stream).unwrap();
    return bitfield_message.id == PeerMessageId::Bitfield && bitfield_message.payload.len() == 1;
}

fn ask_for_piece(piece_index: u32, stream: &mut TcpStream, meta: Metainfo) -> Vec<u8> {
    let request = PeerMessage::request(piece_index, 0, meta.info.piece_length as u32);
    send_message(stream, &request).unwrap();

    let response: PeerMessage = wait_for_message(stream).unwrap();
    response.payload[8..].to_vec()
}

fn ask_for_pieces(stream: &mut TcpStream, meta: &Metainfo) -> Vec<Vec<u8>> {
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    pieces.push(ask_for_piece(0, stream, meta.clone()));
    pieces.push(ask_for_piece(1, stream, meta.clone()));
    pieces.push(ask_for_piece(2, stream, meta.clone()));

    pieces
}

#[test]
fn server_integration_test_ask_for_small_pieces() {
    let peer_id: Vec<u8> = rand::thread_rng().gen::<[u8; 20]>().to_vec();
    let port: u16 = 6002;
    let (pieces, info_hash) = init_pieces();
    let expected_pieces = pieces.clone();
    let meta: Metainfo = get_metainfo(pieces, info_hash);
    let meta_clone = meta.clone();
    let peer_id_clone = peer_id.clone();

    let config: Config = Config {
        listen_port: port,
        log_path: "./log".to_string(),
        download_path: "./downloads".to_string(),
        persist_pieces: true,
    };

    let client_info: ClientInfo = ClientInfo {
        peer_id: peer_id.clone().try_into().unwrap(),
        metainfo: meta_clone.clone(),
        config,
    };

    let server: Server = Server::run(
        peer_id,
        meta.clone(),
        port,
        std::time::Duration::from_secs(2),
        "./downloads/test_server/pieces",
        TrackerService::new(client_info),
    );
    let mut socket: TcpStream;
    loop {
        if let Ok(s) = TcpStream::connect("127.0.0.1:6002") {
            socket = s;
            break;
        }
    }

    let init_result: bool = init_connection(&mut socket, &meta_clone, &peer_id_clone);
    let pieces_data: Vec<Vec<u8>> = ask_for_pieces(&mut socket, &meta_clone);
    server.stop().unwrap();

    assert!(init_result);
    assert_eq!(pieces_data, expected_pieces);
}

fn ask_for_block(block_no: u32, stream: &mut TcpStream) -> Vec<u8> {
    let block_size: u32 = 8;
    let request = PeerMessage::request(0, block_size * block_no, block_size);
    send_message(stream, &request).unwrap();

    let response: PeerMessage = wait_for_message(stream).unwrap();
    response.payload[8..].to_vec()
}

//Returns a vector with each block
fn ask_for_piece_in_blocks(stream: &mut TcpStream) -> Vec<Vec<u8>> {
    let mut blocks: Vec<Vec<u8>> = Vec::new();
    for block_no in 0..3 {
        blocks.push(ask_for_block(block_no, stream));
    }

    blocks
}

#[test]
fn server_integration_test_ask_for_blocks_single_piece() {
    let peer_id: Vec<u8> = rand::thread_rng().gen::<[u8; 20]>().to_vec();
    let port: u16 = 6001;

    std::fs::create_dir_all("./tests/test_server/pieces").unwrap();
    let mut file = File::create("./tests/test_server/pieces/0").unwrap();
    let mut piece: Vec<u8> = Vec::new();

    // This time, we ask for 3 blocks of 8 bytes each
    for i in 0..24 {
        piece.push(i as u8);
    }

    file.write_all(piece.as_slice()).unwrap();
    file.flush().unwrap();

    let mut pieces: Vec<Vec<u8>> = Vec::new();
    pieces.push(piece.clone());
    let meta: Metainfo = get_metainfo(pieces, sha1_of(&piece));
    let meta_clone = meta.clone();
    let peer_id_clone = peer_id.clone();

    let client_info = ClientInfo {
        config: Config::from_path("tests/test_config.txt").unwrap(),
        peer_id: generate_peer_id(),
        metainfo: meta_clone.clone(),
    };

    let server: Server = Server::run(
        peer_id,
        meta,
        port,
        Duration::from_secs(4),
        "./tests/test_server/pieces",
        TrackerService::new(client_info),
    );
    let mut socket: TcpStream;
    loop {
        if let Ok(s) = TcpStream::connect("127.0.0.1:6001") {
            socket = s;
            break;
        }
    }

    let init_result: bool = init_connection(&mut socket, &meta_clone, &peer_id_clone);
    let blocks: Vec<Vec<u8>> = ask_for_piece_in_blocks(&mut socket);
    server.stop().unwrap();

    // concatenate all items in blocks vector into a single Vec<u8> with all bytes joined together
    let mut received_piece: Vec<u8> = Vec::new();
    for block in blocks {
        received_piece.extend_from_slice(block.as_slice());
    }

    assert!(init_result);
    assert_eq!(piece, received_piece);
}
