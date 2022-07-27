use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::server::announce::parse_request_from_params;
use crate::server::announce::AnnounceManager;
use crate::server::announce::AnnounceRequest;
use crate::server::errors::AnnounceError;
use std::collections::HashMap;

pub struct AnnounceController;

impl AnnounceController {
    pub fn handle_announce(
        http_service: Box<dyn IHttpService>,
        request: HttpGetRequest,
        announce_manager: AnnounceManager,
    ) -> Result<(), AnnounceError> {
        let params: HashMap<String, String> = request.params;
        let announce_request: AnnounceRequest = parse_request_from_params(params)?;
        announce_manager.announce(announce_request, http_service);
        Ok(())
    }
}
