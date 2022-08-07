use bittorrent_rustico::logger::CustomLogger;
use std::thread;
use tracker::aggregator::Aggregator;
use tracker::application_constants::STORE_DAYS;
use tracker::application_constants::{LISTEN_PORT, LOCALHOST};
use tracker::http::HttpServiceFactory;
use tracker::metrics::new_metrics;
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

    let (metrics_sender, mut metrics_worker) = new_metrics(STORE_DAYS);

    let aggregator: Aggregator = match Aggregator::start() {
        Ok(aggregator) => aggregator,
        Err(error) => {
            LOGGER.error(format!("Error creating aggregator: {:?}", error));
            return;
        }
    };

    let mut aggregator_worker = aggregator.worker;
    let handle_metrics = thread::spawn(move || {
        let _ = metrics_worker.listen();
    });
    let handle_aggregator = thread::spawn(move || {
        let _ = aggregator_worker.listen(metrics_sender);
    });
    let handle_tracker = thread::spawn(move || {
        let _ = TrackerServer::listen(Box::new(http_service_factory), aggregator.sender);
    });

    handle_tracker
        .join()
        .expect("Could not join tracker server thread");
    handle_aggregator
        .join()
        .expect("Could not join aggregator thread");
    handle_metrics
        .join()
        .expect("Could not join metrics thread");
}
