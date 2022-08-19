use super::AnnounceMessage;
use super::AnnounceRequest;
use crate::server::announce::TrackerResponse;
use bittorrent_rustico::logger::CustomLogger;
use std::sync::mpsc::RecvError;
use std::sync::mpsc::Sender;

const LOGGER: CustomLogger = CustomLogger::init("AnnounceManager");

#[derive(Clone, Debug)]
pub struct AnnounceManager {
    sender: Sender<AnnounceMessage>,
}

impl AnnounceManager {
    pub fn new(sender: Sender<AnnounceMessage>) -> Self {
        AnnounceManager { sender }
    }

    pub fn announce_and_get_response(
        &self,
        announce_request: AnnounceRequest,
        tracker_interval_seconds: u32,
    ) -> Result<TrackerResponse, RecvError> {
        LOGGER.info(format!(
            "request: {:?} - {:?}",
            announce_request.peer_id, announce_request.event
        ));
        let (sender, receiver) = std::sync::mpsc::channel();
        let _ = self.sender.send(AnnounceMessage::Announce(
            announce_request,
            sender,
            tracker_interval_seconds,
        ));

        let response: TrackerResponse = receiver.recv()?;
        LOGGER.info(format!(
            "response: peers: {:?} - seeders: {:?}",
            response.peers.len(),
            response.complete
        ));

        Ok(response)
    }
}
