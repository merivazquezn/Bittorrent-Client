use super::super::constants::STATS_ENDPOINT;
use crate::http::HttpError;
use crate::http::HttpGetRequest;
use crate::http::IHttpService;
use std::fs;

pub struct StaticResourceController;

impl StaticResourceController {
    pub fn serve_static_resources(
        mut http_service: Box<dyn IHttpService>,
        request: HttpGetRequest,
    ) -> Result<(), HttpError> {
        let filename = if request.path.contains(STATS_ENDPOINT) || request.path.is_empty() {
            "index.html".to_string()
        } else {
            request.path
        };

        let contents_result = fs::read(format!(
            "{}{}{}",
            std::env::current_dir().unwrap().display(),
            "/frontend/build/",
            filename
        ));
        if contents_result.is_err() {
            return http_service.send_not_found();
        }
        let content: Vec<u8> = contents_result.unwrap();

        // based on the extension, write the correct content type to the header
        let content_type = if filename.ends_with(".html") {
            "text/html"
        } else if filename.ends_with(".css") {
            "text/css"
        } else if filename.ends_with(".js") {
            "application/javascript"
        } else if filename.ends_with(".png") {
            "image/png"
        } else if filename.ends_with(".jpg") {
            "image/jpeg"
        } else if filename.ends_with(".gif") {
            "image/gif"
        } else if filename.ends_with(".svg") {
            "image/svg+xml"
        } else if filename.ends_with(".ico") {
            "image/x-icon"
        } else {
            "text/plain"
        };

        http_service.send_ok_response(content, content_type.to_string())?;

        Ok(())
    }
}
