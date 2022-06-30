use bittorrent_rustico::client::*;
use bittorrent_rustico::config::*;
use bittorrent_rustico::constants::*;
use bittorrent_rustico::download_manager::join_all_pieces;
use bittorrent_rustico::metainfo::*;
use bittorrent_rustico::peer::*;
use bittorrent_rustico::ui::*;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Read;

mod mock_service_creation;
use mock_service_creation::*;

fn get_mock_peer_list() -> Vec<Peer> {
    let peer_1 = Peer {
        ip: String::from("0.0.0.0"),
        port: 0,
        peer_id: vec![0],
        peer_message_service_provider: mock_peer_message_service_1,
    };
    let peer_2 = Peer {
        ip: String::from("1.1.1.1"),
        port: 0,
        peer_id: vec![1],
        peer_message_service_provider: mock_peer_message_service_2,
    };
    let peer_3 = Peer {
        ip: String::from("2.2.2.2"),
        port: 0,
        peer_id: vec![2],
        peer_message_service_provider: mock_peer_message_service_3,
    };

    vec![peer_1, peer_2, peer_3]
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

#[test]
fn integration_test() {
    pretty_env_logger::init();

    let mut file = Vec::new();

    for _i in 0..BLOCK_SIZE {
        file.push(1);
    }
    for _i in 0..BLOCK_SIZE {
        file.push(2);
    }
    for _i in 0..BLOCK_SIZE {
        file.push(3);
    }

    let info = Info {
        piece_length: BLOCK_SIZE,
        pieces: get_pieces_hash_from_bytes(&file),
        name: String::from("integration_test"),
        length: file.len() as u64,
        files: None,
    };
    let metainfo = Metainfo {
        announce: String::from("mock_url"),
        info_hash: vec![],
        info,
    };

    let peers = get_mock_peer_list();

    let client_info = ClientInfo {
        config: Config::from_path("tests/test_config.txt").unwrap(),
        peer_id: generate_peer_id(),
        metainfo,
    };
    let client: TorrentClient = TorrentClient::new(&client_info, UIMessageSender::no_ui()).unwrap();
    client.run_with_peers(peers, client_info).unwrap();
    join_all_pieces(3, "entire_download", "tests/downloads").unwrap();
    // assert file tests/entire_download equals file
    let mut entire_file: File = File::open("tests/downloads/entire_download").unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let _ = entire_file.read_to_end(&mut buf).unwrap();
    assert_eq!(file, buf);
}
