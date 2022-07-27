use super::AnnounceMessage;
use super::AnnounceRequest;
use crate::http::IHttpService;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct AnnounceManager {
    sender: Sender<AnnounceMessage>,
}

impl AnnounceManager {
    pub fn new(sender: Sender<AnnounceMessage>) -> Self {
        AnnounceManager { sender }
    }

    pub fn announce(&self, announce_request: AnnounceRequest, http_service: Box<dyn IHttpService>) {
        let _ = self
            .sender
            .send(AnnounceMessage::Announce(announce_request, http_service));
    }
}
