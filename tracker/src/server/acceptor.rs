use super::constants::POOL_WORKERS;
use super::errors::TrackerError;
use super::static_resource_controller::StaticResourceController;
use super::types::TrackerEndpoint;
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

        loop {
            LOGGER.info_str("Estoy esperando una conexion...");
            let mut http_service: Box<dyn IHttpService> =
                http_service_factory.get_new_connection()?;

            LOGGER.info_str("Llego una conexion");
            pool.execute(move || match http_service.parse_request() {
                Ok(request) => {
                    LOGGER.info(format!("Request: {:?}", request));
                    if let Err(e) = Self::handle_incoming_connection(http_service, request) {
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
    ) -> Result<(), TrackerError> {
        let endpoint: TrackerEndpoint = parse_path(&request.path);
        LOGGER.info(format!("Lei endpoint: {:?}", endpoint));
        match endpoint {
            TrackerEndpoint::StaticResource => {
                Ok(StaticResourceController::handle(http_service, request)?)
            }
            _ => {
                unimplemented!();
            }
        }
    }
}
