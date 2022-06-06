use super::errors::HttpsServiceError;

pub trait IHttpService {
    fn get(&mut self, path: &str, query_params: &str) -> Result<Vec<u8>, HttpsServiceError>;
}
