use std::time::Duration;
use tracker::server::TrackerServer;

fn main() {
    let tracker_server = TrackerServer::new(Duration::from_secs(3));
    tracker_server.stop().unwrap();
}
