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
    /// Creates a new AnnounceManager sender
    pub fn new(sender: Sender<AnnounceMessage>) -> Self {
        AnnounceManager { sender }
    }

    /// Sends a update message to the AnnounceManager, which will
    /// update all the active peers entries for all torrents.
    pub fn update(&self) {
        println!("sending update to announce manager");
        let _ = self.sender.send(AnnounceMessage::Update);
    }

    /// Sends a announce message to the AnnounceManager, which will
    /// Build the response for the announce request.
    /// This response contains the list of peers that are currently
    /// active for the torrent.
    /// If the torrent doesnot exist, it will create a new torrent entry, but
    /// the active peers response will be empty
    ///
    /// It returns an error if sending the message through the channel fails
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
