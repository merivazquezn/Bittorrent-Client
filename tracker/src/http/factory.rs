use super::HttpError;
use super::{HttpService, IHttpService};
use std::net::TcpListener;

pub trait IHttpServiceFactory {
    fn get_new_connection(&self) -> Result<Box<dyn IHttpService>, HttpError>;
}

pub struct HttpServiceFactory {
    tcp_listener: TcpListener,
}

impl HttpServiceFactory {
    pub fn new(tcp_listener: TcpListener) -> HttpServiceFactory {
        HttpServiceFactory { tcp_listener }
    }
}

impl IHttpServiceFactory for HttpServiceFactory {
    fn get_new_connection(&self) -> Result<Box<dyn IHttpService>, HttpError> {
        match self.tcp_listener.accept() {
            Ok((stream, _)) => Ok(Box::new(HttpService::from_stream(stream))),
            Err(_) => Err(HttpError::HttpError(
                "Could not accept connection".to_string(),
            )),
        }
    }
}
