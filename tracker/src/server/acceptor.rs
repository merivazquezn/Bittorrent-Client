use super::announce::new_announce_manager;
use super::announce::AnnounceManager;
use super::constants::POOL_WORKERS;
use super::controllers::AnnounceController;
use super::controllers::MetricsController;
use super::controllers::StaticResourceController;
use super::endpoints::TrackerEndpoint;
use super::errors::TrackerError;
use super::utils::parse_path;
use crate::aggregator::AggregatorSender;
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
        aggregator: AggregatorSender,
        metrics: MetricsSender,
        threads: usize,
        receiver: std::sync::mpsc::Receiver<()>,
    ) -> Result<(), TrackerError> {
        let pool: ThreadPool = ThreadPool::new(threads)?;
        let (announce_manager_sender, announce_manager_worker) = new_announce_manager(aggregator);
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
                    LOGGER.info(format!("Request: {:?}", request));
                    if let Err(e) = Self::handle_incoming_connection(
                        http_service,
                        request,
                        announce_manager,
                        metrics_clone,
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
            )?),
            TrackerEndpoint::Metrics => Ok(MetricsController::handler_metrics(
                http_service,
                request,
                metrics,
            )?),
        }
    }
}
