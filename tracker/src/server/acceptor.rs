use super::announce::new_announce_manager;
use super::announce::AnnounceManager;
use super::constants::POOL_WORKERS;
use super::controllers::AnnounceController;
use super::controllers::StaticResourceController;
use super::endpoints::TrackerEndpoint;
use super::errors::TrackerError;
use super::utils::parse_path;
use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::http::IHttpServiceFactory;
use bittorrent_rustico::logger::CustomLogger;
use bittorrent_rustico::server::ThreadPool;

const LOGGER: CustomLogger = CustomLogger::init("Acceptor");

pub struct TrackerServer;

impl TrackerServer {
    pub fn listen(http_service_factory: Box<dyn IHttpServiceFactory>) -> Result<(), TrackerError> {
        let pool: ThreadPool = ThreadPool::new(POOL_WORKERS)?;
        let (announce_manager_sender, announce_manager_worker) = new_announce_manager();
        std::thread::spawn(move || {
            let _ = announce_manager_worker.listen();
        });

        loop {
            LOGGER.info_str("Server waiting for connection...");
            let mut http_service: Box<dyn IHttpService> =
                http_service_factory.get_new_connection()?;

            LOGGER.info_str("Incoming connection");
            let announce_manager: AnnounceManager = announce_manager_sender.clone();

            pool.execute(move || match http_service.parse_request() {
                Ok(request) => {
                    LOGGER.info(format!("Request: {:?}", request));
                    if let Err(e) =
                        Self::handle_incoming_connection(http_service, request, announce_manager)
                    {
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
            _ => {
                unimplemented!();
            }
        }
    }
}
