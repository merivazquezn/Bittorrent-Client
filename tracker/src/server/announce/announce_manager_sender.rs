use super::AnnounceMessage;
use super::AnnounceRequest;
use crate::server::announce::TrackerResponse;
use std::sync::mpsc::RecvError;
use std::sync::mpsc::Sender;

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
        println!("request: {:?}", announce_request);
        let (sender, receiver) = std::sync::mpsc::channel();
        let _ = self.sender.send(AnnounceMessage::Announce(
            announce_request,
            sender,
            tracker_interval_seconds,
        ));

        let response: TrackerResponse = receiver.recv()?;
        println!("{:?}", response);

        Ok(response)
    }
}
