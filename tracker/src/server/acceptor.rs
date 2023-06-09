use super::announce::AnnounceManager;
use super::announce::AnnounceManagerWorker;
use super::controllers::AnnounceController;
use super::controllers::MetricsController;
use super::controllers::StaticResourceController;
use super::endpoints::TrackerEndpoint;
use super::errors::TrackerError;
use super::utils::parse_path;
use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::http::IHttpServiceFactory;
use crate::metrics::MetricsSender;
use bittorrent_rustico::logger::CustomLogger;
use bittorrent_rustico::server::ThreadPool;

const LOGGER: CustomLogger = CustomLogger::init("Acceptor");

pub struct TrackerServer;

impl TrackerServer {
    pub fn listen(
        mut http_service_factory: Box<dyn IHttpServiceFactory>,
        metrics: MetricsSender,
        threads: usize,
        tracker_interval_seconds: u32,
        receiver: std::sync::mpsc::Receiver<()>,
        announce_manager_sender: AnnounceManager,
        announce_manager_worker: AnnounceManagerWorker,
    ) -> Result<(), TrackerError> {
        let pool: ThreadPool = ThreadPool::new(threads)?;
        std::thread::spawn(move || {
            let _ = announce_manager_worker.listen();
        });

        loop {
            LOGGER.info_str("Server waiting for connection...");

            if receiver.try_recv().is_ok() {
                let _ = pool.stop();
                return Ok(());
            }

            let mut http_service: Box<dyn IHttpService> =
                match http_service_factory.get_new_connection() {
                    Ok(http_service) => http_service,
                    Err(error) => {
                        LOGGER.error(format!("Error creating http service: {:?}", error));
                        continue;
                    }
                };

            LOGGER.info_str("Incoming connection");
            let announce_manager: AnnounceManager = announce_manager_sender.clone();

            let metrics_clone = metrics.clone();
            pool.execute(move || match http_service.parse_request() {
                Ok(request) => {
                    if let Err(e) = Self::handle_incoming_connection(
                        http_service,
                        request,
                        announce_manager,
                        metrics_clone,
                        tracker_interval_seconds,
                    ) {
                        LOGGER.info(format!("Error handling incoming connection: {:?}", e));
                    }
                }
                Err(error) => {
                    LOGGER.info(format!("Error parsing request: {:?}", error));
                }
            });
        }
    }

    fn handle_incoming_connection(
        http_service: Box<dyn IHttpService>,
        request: HttpGetRequest,
        announce_manager: AnnounceManager,
        metrics: MetricsSender,
        tracker_interval_seconds: u32,
    ) -> Result<(), TrackerError> {
        let endpoint: TrackerEndpoint = parse_path(&request.path);
        LOGGER.info(format!("Received endpoint: {:?}", endpoint));
        match endpoint {
            TrackerEndpoint::StaticResource => Ok(
                StaticResourceController::serve_static_resources(http_service, request)?,
            ),
            TrackerEndpoint::Announce => Ok(AnnounceController::handle_announce(
                http_service,
                request,
                announce_manager,
                tracker_interval_seconds,
            )?),
            TrackerEndpoint::Metrics => Ok(MetricsController::handler_metrics(
                http_service,
                request,
                metrics,
            )?),
            TrackerEndpoint::Torrents => {
                Ok(MetricsController::get_torrents(http_service, metrics)?)
            }
        }
    }
}
