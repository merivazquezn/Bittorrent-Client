use super::types::Peer;
use super::AnnounceMessage;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

pub struct AnnounceManagerWorker {
    _peers_by_torrent: HashMap<Vec<u8>, Vec<Peer>>,
    receiver: Receiver<AnnounceMessage>,
}

impl AnnounceManagerWorker {
    pub fn new(receiver: Receiver<AnnounceMessage>) -> Self {
        AnnounceManagerWorker {
            _peers_by_torrent: HashMap::new(),
            receiver,
        }
    }

    pub fn listen(self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            match message {
                AnnounceMessage::Announce(_announce_request, _http_service) => {
                    println!("Received announce message");
                }
            }
        }
    }
}
