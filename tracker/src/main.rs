use pretty_env_logger;
use std::sync::mpsc::channel;
use std::thread;
use tracker::server::TrackerServer;

fn main() {
    pretty_env_logger::init();
    let (tx, rx) = channel();
    let handle = thread::spawn(move || {
        let res = rx.recv().unwrap();
    });

    let tracker_server = TrackerServer::new(tx);

    tracker_server.handle.join().unwrap().unwrap();
}
