use super::super::announce::utils;
use crate::http::HttpError;
use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use crate::server::announce::parse_request_from_params;
use crate::server::announce::AnnounceManager;
use crate::server::announce::AnnounceRequest;
use crate::server::announce::TrackerResponse;
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
        let announce_request: AnnounceRequest =
            parse_request_from_params(params, http_service.get_client_address())?;
        let response: TrackerResponse =
            announce_manager.announce_and_get_response(announce_request)?;

        Self::send_response(http_service, response)?;
        Ok(())
    }

    fn send_response(
        mut http_service: Box<dyn IHttpService>,
        response: TrackerResponse,
    ) -> Result<(), HttpError> {
        let response_bytes: Vec<u8> = utils::get_response_bytes(response);
        http_service.send_ok_response(response_bytes, "application/octet-stream".to_string())
    }
}
