use std::collections::HashMap;
use std::thread;
use tracker::aggregator::Aggregator;
use tracker::http::IHttpServiceFactory;
use tracker::metrics::new_metrics;
use tracker::server::TrackerServer;
mod mocks;
use mocks::*;
use tracker::server::announce::utils::get_response_bytes;
use tracker::server::announce::TrackerResponse;

fn setup() {
    pretty_env_logger::init();
}
#[test]
fn first_peer_connection_should_return_empty_peer_list() {
    setup();

    let first_connection = create_mock_connection(
        0,
        0,
        0,
        "b111813ce60f42919734823df5ec20bd1e04e7f7",
        "b000000000000000000000000000000000000000",
        0,
        "0.0.0.0:8080",
    );

    run_mock_server(vec![first_connection]);

    let content = std::fs::read("./tests/announce/0/content").unwrap();

    let expected_tracker_response = TrackerResponse {
        interval_in_seconds: 120,
        complete: 0,
        incomplete: 0,
        tracker_id: "Polleria Rustiseria Tracker ID :)".to_string(),
        peers: vec![],
    };
    let expected = get_response_bytes(expected_tracker_response);

    assert_eq!(content, expected, "contents of response do not match");
}
