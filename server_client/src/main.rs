use bittorrent_rustico::metainfo::{self, Info, Metainfo};
use bittorrent_rustico::peer::IPeerMessageServiceError;
use bittorrent_rustico::peer::{PeerMessage, PeerMessageId};
use bittorrent_rustico::server::Server;
use rand::Rng;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn generate_peer_id() -> [u8; 20] {
    rand::thread_rng().gen::<[u8; 20]>()
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

fn sha1_of(vec: &[u8]) -> Vec<u8> {
    let mut hasher = Sha1::new();
    hasher.update(vec);
    hasher.finalize().to_vec()
}

// Returns a vector of the pieces each piece
fn init_pieces() -> (Vec<Vec<u8>>, Vec<u8>) {
    let mut pieces: Vec<Vec<u8>> = Vec::new();
    let mut joined_pieces: Vec<u8> = Vec::new();

    let mut file_0 = File::create("../downloads/pieces/0").unwrap();
    let mut buf_0: Vec<u8> = Vec::new();
    for i in 0..8 {
        buf_0.push(i as u8);
        joined_pieces.push(i as u8);
    }
    pieces.push(buf_0.clone());

    file_0.write_all(buf_0.as_slice()).unwrap();

    let mut file_1 = File::create("../downloads/pieces/1").unwrap();
    let mut buf_1: Vec<u8> = Vec::new();
    for i in 0..8 {
        buf_1.push((8 - i) as u8);
        joined_pieces.push((8 - i) as u8)
    }
    pieces.push(buf_1.clone());
    file_1.write_all(buf_1.as_slice()).unwrap();

    let mut file_2 = File::create("../downloads/pieces/2").unwrap();
    let mut buf_2: Vec<u8> = Vec::new();
    for _ in 0..8 {
        buf_2.push(3 as u8);
        joined_pieces.push(3 as u8);
    }
    pieces.push(buf_2.clone());
    file_2.write_all(buf_2.as_slice()).unwrap();

    (pieces, sha1_of(&joined_pieces))
}

fn create_handshake_message(info_hash: &[u8], peer_id: &[u8]) -> Vec<u8> {
    let mut handshake_message = Vec::new();
    handshake_message.extend_from_slice(&[19]);
    handshake_message.extend_from_slice(b"BitTorrent protocol");
    handshake_message.extend_from_slice(&[0u8; 8]);
    handshake_message.extend_from_slice(info_hash);
    handshake_message.extend_from_slice(peer_id);
    handshake_message
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
    println!("Client: received message of length {}", message_length);

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

fn init_connection(stream: &mut TcpStream, meta: &Metainfo, peer_id: &Vec<u8>) {
    let handshake_message: Vec<u8> = create_handshake_message(&meta.info_hash, &peer_id);
    stream.write_all(&handshake_message).unwrap();
    let mut handshake_response = [0u8; 68];
    stream.read_exact(&mut handshake_response).unwrap();

    let unchocke_message: PeerMessage = wait_for_message(stream).unwrap();
    if unchocke_message.id == PeerMessageId::Unchoke {
        println!("Client received unchoke message from server");
    } else {
        println!("Error: Client should have received an unchoke message");
    }

    println!("Client waiting for bitfield message...");
    let bitfield_message: PeerMessage = wait_for_message(stream).unwrap();

    if bitfield_message.id == PeerMessageId::Bitfield {
        println!(
            "Client received bitfield from server: {:?}",
            bitfield_message
        );
    } else {
        println!("Error: should have received bitfield message form server");
    }

    println!("Client: handshake and initial messages successful");
}

fn ask_for_piece(piece_index: u32, stream: &mut TcpStream, meta: Metainfo) -> Vec<u8> {
    let request = PeerMessage::request(piece_index, 0, meta.info.piece_length as u32);
    println!("Client: About to send {:?} to server", request);
    send_message(stream, &request).unwrap();
    println!("Client sent message {:?} to server", request);
    let response: PeerMessage = wait_for_message(stream).unwrap();
    response.payload[8..].to_vec()
}

fn ask_for_pieces(stream: &mut TcpStream, meta: &Metainfo) -> Vec<Vec<u8>> {
    let mut pieces: Vec<Vec<u8>> = Vec::new();

    println!("Client about to start asking server for pieces");

    for i in 0..3 {
        pieces.push(ask_for_piece(i, stream, meta.clone()));
    }

    pieces
}

fn piece_is_valid(data: &Vec<u8>, expected: Vec<u8>) -> bool {
    println!("\nExpected: {:?}\nActual: {:?}", expected, data);
    sha1_of(&data) == sha1_of(&expected)
}

fn run_test_client_server() {
    let peer_id: Vec<u8> = generate_peer_id().to_vec();
    let port: u16 = 6881;
    let (pieces, info_hash) = init_pieces();
    let expected_pieces = pieces.clone();
    let meta: Metainfo = get_metainfo(pieces, info_hash);
    let meta_clone = meta.clone();
    let peer_id_clone = peer_id.clone();

    println!("Trying to connect to server at: 127.0.0.1:6681");
    let server: Server = Server::run(peer_id, meta, port, Duration::from_secs(2));
    let mut stream: TcpStream;
    loop {
        if let Ok(s) = TcpStream::connect("127.0.0.1:6881") {
            stream = s;
            break;
        }
    }
    println!("Client connected to server");

    init_connection(&mut stream, &meta_clone, &peer_id_clone);
    let pieces_data: Vec<Vec<u8>> = ask_for_pieces(&mut stream, &meta_clone);

    for (index, piece_data) in pieces_data.iter().enumerate() {
        let expected_piece = expected_pieces.get(index).unwrap();
        if piece_is_valid(piece_data, expected_piece.to_vec()) {
            println!("Piece {} was validated correctly\n", index);
        } else {
            println!("Piece {} hash is invalid\n", index);
        }
    }

    match server.stop() {
        Ok(()) => println!("Program ended successfully"),
        Err(e) => println!("Server stopping error: {}", e),
    }
}

fn main() {
    run_test_client_server();
}
