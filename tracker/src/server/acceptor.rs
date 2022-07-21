use super::constants::POOL_WORKERS;
use super::errors::TrackerError;
use super::stats::StatsRequestHandler;
use super::types::TrackerEndpoint;
use super::utils::parse_path;
use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::http::IHttpServiceFactory;
use bittorrent_rustico::server::ThreadPool;

pub struct TrackerServer;

impl TrackerServer {
    pub fn listen(http_service_factory: Box<dyn IHttpServiceFactory>) -> Result<(), TrackerError> {
        let pool: ThreadPool = ThreadPool::new(POOL_WORKERS)?;

        loop {
            println!("Estoy esperando una conexion...");
            let mut http_service: Box<dyn IHttpService> =
                http_service_factory.get_new_connection()?;

            println!("Llego una conexion");
            pool.execute(move || match http_service.parse_request() {
                Ok(request) => {
                    println!("Request: {:?}", request);
                    if let Err(e) = Self::handle_incoming_connection(http_service, request) {
                        println!("Error handling incoming connection: {:?}", e);
                    }
                }
                Err(error) => {
                    println!("Error parsing request: {:?}", error);
                }
            });
        }
    }

    fn handle_incoming_connection(
        mut http_service: Box<dyn IHttpService>,
        request: HttpGetRequest,
    ) -> Result<(), TrackerError> {
        let endpoint: TrackerEndpoint = parse_path(&request.path);
        println!("Lei endpoint: {:?}", endpoint);
        match endpoint {
            TrackerEndpoint::Stats => Ok(StatsRequestHandler::handle(http_service, request)?),
            _ => {
                http_service.send_not_found()?;
                Err(TrackerError::InvalidEndpoint(request.path))
            }
        }
    }
}
