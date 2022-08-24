use bittorrent_rustico::logger::CustomLogger;
use std::thread;
use tracker::aggregator::Aggregator;
use tracker::aggregator::Timer;
use tracker::application_constants::STORE_DAYS;
use tracker::application_constants::{LISTEN_PORT, LOCALHOST, RECOVER_METRICS_FLAG};
use tracker::http::HttpServiceFactory;
use tracker::metrics::new_metrics;
use tracker::server::announce::new_announce_manager;
use tracker::server::TrackerServer;

const TRACKER_INTERVAL_IN_SECONDS: u32 = 20;
const LOGGER: CustomLogger = CustomLogger::init("Acceptor");

fn bind_server() -> std::net::TcpListener {
    std::net::TcpListener::bind(format!("{}:{}", LOCALHOST, LISTEN_PORT))
        .expect("Could not bind to port")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut should_recover_metrics: bool = false;
    for arg in args {
        if arg == RECOVER_METRICS_FLAG {
            println!("Recovering metrics...");
            should_recover_metrics = true;
        }
    }

    pretty_env_logger::init();
    LOGGER.info(format!(
        "Tracker listening at {}:{}",
        LOCALHOST, LISTEN_PORT
    ));
    let http_service_factory = HttpServiceFactory::new(bind_server());

    let timer = Timer::new();

    let (metrics_sender, mut metrics_worker) = new_metrics(STORE_DAYS, should_recover_metrics);

    let aggregator: Aggregator = match Aggregator::start(timer.sender.clone()) {
        Ok(aggregator) => aggregator,
        Err(error) => {
            LOGGER.error(format!("Error creating aggregator: {:?}", error));
            return;
        }
    };

    let mut aggregator_worker = aggregator.worker;
    let _ = thread::spawn(move || {
        let _ = metrics_worker.listen();
    });
    let metrics = metrics_sender.clone();
    let _ = thread::spawn(move || {
        let _ = aggregator_worker.listen(metrics);
    });
    let (announce_manager_sender, announce_manager_worker) =
        new_announce_manager(aggregator.sender.clone(), TRACKER_INTERVAL_IN_SECONDS);
    let announce_manager_sender_clone = announce_manager_sender.clone();
    let (_, tracker_receiver) = std::sync::mpsc::channel();
    let handle_tracker = thread::spawn(move || {
        let _ = TrackerServer::listen(
            Box::new(http_service_factory),
            metrics_sender,
            100,
            TRACKER_INTERVAL_IN_SECONDS,
            tracker_receiver,
            announce_manager_sender_clone,
            announce_manager_worker,
        );
    });

    let _ = std::thread::spawn(move || {
        timer
            .worker
            .start(aggregator.sender, announce_manager_sender)
    });

    handle_tracker
        .join()
        .expect("Could not join tracker server thread");
}
