use super::errors::HttpsConnectionError;

pub trait HttpService {
    fn get(&mut self, path: &str, query_params: &str) -> Result<Vec<u8>, HttpsConnectionError>;
}
