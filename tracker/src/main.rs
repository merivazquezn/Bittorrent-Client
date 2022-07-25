use bittorrent_rustico::logger::CustomLogger;
use std::thread;
use tracker::application_constants::{LISTEN_PORT, LOCALHOST};
use tracker::http::HttpServiceFactory;
use tracker::server::TrackerServer;

const LOGGER: CustomLogger = CustomLogger::init("Acceptor");

fn bind_server() -> std::net::TcpListener {
    std::net::TcpListener::bind(format!("{}:{}", LOCALHOST, LISTEN_PORT))
        .expect("Could not bind to port")
}

fn main() {
    pretty_env_logger::init();
    LOGGER.info(format!(
        "Tracker escuchando en {}:{}",
        LOCALHOST, LISTEN_PORT
    ));
    let http_service_factory = HttpServiceFactory::new(bind_server());
    let handle = thread::spawn(|| {
        let _ = TrackerServer::listen(Box::new(http_service_factory));
    });

    handle.join().expect("Could not join tracker server thread");
}
