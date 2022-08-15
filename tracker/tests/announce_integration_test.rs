mod mocks;
use bittorrent_rustico::bencode;
use mocks::*;
use tracker::server::announce::utils::get_response_bytes;
use tracker::server::announce::{Peer, TrackerResponse};

fn setup() {
    pretty_env_logger::init();
}
#[test]
fn first_peer_connection_should_return_empty_peer_list() {
    setup();
    let test_name = "first_connection";

    let first_connection = create_mock_connection(
        0,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000000",
        test_name,
        0,
        "0.0.0.0:8080",
    );

    run_mock_server(vec![first_connection], 120, None);

    let expected_tracker_response = TrackerResponse {
        interval_in_seconds: 120,
        complete: 0,
        incomplete: 0,
        tracker_id: "Polleria Rustiseria Tracker ID :)".to_string(),
        peers: vec![],
    };
    let expected = get_response_bytes(expected_tracker_response);

    assert_eq!(
        get_content_from_test(test_name, 0),
        expected,
        "contents of response do not match"
    );
}

#[test]
fn second_peer_obtains_first_peer_in_list() {
    let test_name = "two_connections";

    let first_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000000",
        test_name,
        0,
        "0.0.0.0:8080",
    );

    let second_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000001",
        test_name,
        1,
        "0.0.0.1:8080",
    );

    run_mock_server(vec![first_connection, second_connection], 120, None);

    let expected_tracker_response = TrackerResponse {
        interval_in_seconds: 120,
        complete: 0,
        incomplete: 1,
        tracker_id: "Polleria Rustiseria Tracker ID :)".to_string(),
        peers: vec![Peer {
            peer_id: "b000000000000000000000000000000000000000"
                .as_bytes()
                .to_vec(),
            ip: "0.0.0.0".to_string(),
            port: 8080,
        }],
    };
    let expected = get_response_bytes(expected_tracker_response);

    assert_eq!(
        get_content_from_test(test_name, 1),
        expected,
        "contents of response do not match"
    );
}

#[test]
fn third_peer_obtains_first_and_second_peer_in_list() {
    let test_name = "three_connections";

    let first_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000000",
        test_name,
        0,
        "0.0.0.0:8080",
    );

    let second_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000001",
        test_name,
        1,
        "0.0.0.1:8080",
    );

    let third_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000002",
        test_name,
        2,
        "0.0.0.2:8080",
    );

    run_mock_server(
        vec![first_connection, second_connection, third_connection],
        120,
        None,
    );

    let expected_tracker_response = TrackerResponse {
        interval_in_seconds: 120,
        complete: 0,
        incomplete: 2,
        tracker_id: "Polleria Rustiseria Tracker ID :)".to_string(),
        peers: vec![
            Peer {
                peer_id: "b000000000000000000000000000000000000000"
                    .as_bytes()
                    .to_vec(),
                ip: "0.0.0.0".to_string(),
                port: 8080,
            },
            Peer {
                peer_id: "b000000000000000000000000000000000000001"
                    .as_bytes()
                    .to_vec(),
                ip: "0.0.0.1".to_string(),
                port: 8080,
            },
        ],
    };
    let expected = get_response_bytes(expected_tracker_response);

    assert_eq!(
        get_content_from_test(test_name, 2),
        expected,
        "contents of response do not match"
    );
}

#[test]
fn first_peer_finishes_download_another_peer_receives_it() {
    let test_name = "first_connection_finishes_download";

    let first_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000000",
        test_name,
        0,
        "0.0.0.0:8080",
    );

    let second_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000001",
        test_name,
        1,
        "0.0.0.1:8080",
    );

    let third_connection = create_mock_connection(
        0,
        50,
        255,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000000",
        test_name,
        2,
        "0.0.0.0:8080",
    );

    let fourth_connection = create_mock_connection(
        235,
        10,
        20,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000002",
        test_name,
        3,
        "0.0.0.2:8080",
    );

    run_mock_server(
        vec![
            first_connection,
            second_connection,
            third_connection,
            fourth_connection,
        ],
        120,
        None,
    );

    let expected_tracker_response = TrackerResponse {
        interval_in_seconds: 120,
        complete: 1,
        incomplete: 1,
        tracker_id: "Polleria Rustiseria Tracker ID :)".to_string(),
        peers: vec![
            Peer {
                peer_id: "b000000000000000000000000000000000000000"
                    .as_bytes()
                    .to_vec(),
                ip: "0.0.0.0".to_string(),
                port: 8080,
            },
            {
                Peer {
                    peer_id: "b000000000000000000000000000000000000001"
                        .as_bytes()
                        .to_vec(),
                    ip: "0.0.0.1".to_string(),
                    port: 8080,
                }
            },
        ],
    };
    let expected = get_response_bytes(expected_tracker_response);

    assert_eq!(
        get_content_from_test(test_name, 3),
        expected,
        "contents of response do not match"
    );
}

#[test]
fn peers_of_different_torrents_dont_know_each_other() {
    let test_name = "different_info_hashes";

    let first_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f6",
        "b000000000000000000000000000000000000000",
        test_name,
        0,
        "0.0.0.0:8080",
    );

    let second_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000001",
        test_name,
        1,
        "0.0.0.1:8080",
    );

    run_mock_server(vec![first_connection, second_connection], 120, None);

    let expected_tracker_response = TrackerResponse {
        interval_in_seconds: 120,
        complete: 0,
        incomplete: 0,
        tracker_id: "Polleria Rustiseria Tracker ID :)".to_string(),
        peers: vec![],
    };
    let expected = get_response_bytes(expected_tracker_response);

    assert_eq!(
        get_content_from_test(test_name, 1),
        expected,
        "contents of response do not match"
    );
}

#[test]
fn peer_becomes_inactive_gets_removed_from_list() {
    let test_name = "peer_becomes_inactive";

    let first_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000000",
        test_name,
        0,
        "0.0.0.0:8080",
    );

    let second_connection = create_mock_connection(
        255,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000001",
        test_name,
        1,
        "0.0.0.1:8080",
    );

    run_mock_server(
        vec![first_connection, second_connection],
        1,
        Some(std::time::Duration::from_millis(2100)),
    );

    let expected_tracker_response = TrackerResponse {
        interval_in_seconds: 1,
        complete: 0,
        incomplete: 0,
        tracker_id: "Polleria Rustiseria Tracker ID :)".to_string(),
        peers: vec![],
    };
    let expected = get_response_bytes(expected_tracker_response);

    assert_eq!(
        get_content_from_test(test_name, 1),
        expected,
        "contents of response do not match"
    );
}
