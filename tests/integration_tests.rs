use bittorrent_rustico::client::*;
use bittorrent_rustico::config::*;
use bittorrent_rustico::constants::*;
use bittorrent_rustico::metainfo::*;
use bittorrent_rustico::peer::*;
use bittorrent_rustico::tracker::*;
use bittorrent_rustico::ui::*;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::Read;
use std::time::Duration;
mod mock_service_creation;
use mock_service_creation::*;

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
    let faulty_peer = Peer {
        ip: String::from("9.9.9.9"),
        port: 0,
        peer_id: vec![99],
        peer_message_service_provider: mock_faulty_peer_message_service,
    };

    vec![
        // vec![peer_0, peer_1, peer_2], /*
        vec![peer_1.clone(), faulty_peer.clone()],
        vec![faulty_peer.clone(), peer_2, peer_0],
        //   vec![faulty_peer.clone(), peer_2],//*/
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

#[test]
fn integration_test() {
    pretty_env_logger::init();

    //create downloads dir
    let downloads_dir_path = "./tests/downloads/pieces";
    std::fs::create_dir_all(downloads_dir_path).unwrap();

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
    let starting_tracker_response = TrackerResponse {
        interval: None::<Duration>,
        peers: tracker_responses[0].to_vec(),
    };
    let client_info = ClientInfo {
        config: Config::from_path("tests/test_config.txt").unwrap(),
        peer_id: generate_peer_id(),
        metainfo,
    };
    let client: TorrentClient = TorrentClient::new(&client_info, UIMessageSender::no_ui()).unwrap();

    let tracker_service = MockTrackerService {
        responses: tracker_responses[1..].to_vec(),
        // responses: vec![],
        response_index: 0,
    };

    client
        .run(
            client_info,
            Box::new(tracker_service),
            starting_tracker_response,
        )
        .unwrap();
    let mut entire_file: File =
        File::open("./tests/downloads/linux_distribution_test.iso/linux_distribution_test.iso")
            .unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let _ = entire_file.read_to_end(&mut buf).unwrap();
    assert_eq!(file, buf);
}
