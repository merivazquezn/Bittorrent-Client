use super::types::Peer;
use super::AnnounceMessage;
use crate::http::IHttpService;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

pub struct AnnounceManagerWorker {
    peers_by_torrent: HashMap<Vec<u8>, Vec<Peer>>,
    receiver: Receiver<AnnounceMessage>,
}

impl AnnounceManagerWorker {
    pub fn new(receiver: Receiver<AnnounceMessage>) -> Self {
        AnnounceManagerWorker {
            peers_by_torrent: HashMap::new(),
            receiver,
        }
    }

    pub fn listen(mut self) -> Result<(), RecvError> {
        loop {
            let message: AnnounceMessage = self.receiver.recv()?;
            match message {
                AnnounceMessage::Announce(announce_request, mut http_service) => {
                    let info_hash: Vec<u8> = announce_request.info_hash.clone();
                    let _peer: Peer = Peer {
                        ip: announce_request.ip,
                        port: announce_request.port,
                        peer_id: announce_request.peer_id.clone(),
                    };

                    if self.torrent_already_exists(&info_hash) {
                        let peers: Vec<Peer> = self.get_peer_from_torrent(&info_hash);
                    } else {
                        self.add_new_torrent_and_send_response(info_hash, http_service);
                    }
                }
            }
        }
    }

    fn add_new_torrent_and_send_response(
        &mut self,
        info_hash: Vec<u8>,
        mut http_service: Box<dyn IHttpService>,
    ) {
        self.peers_by_torrent.insert(info_hash, Vec::new());
        match http_service.send_ok_response(Vec::new(), "nothing yet".to_string()) {
            Ok(()) => println!("Torrent added successfully"),
            Err(err) => println!(
                "Error sending ok response while adding new torrent: {:?}",
                err
            ),
        };
    }

    fn torrent_already_exists(&self, info_hash: &Vec<u8>) -> bool {
        self.peers_by_torrent.contains_key(info_hash)
    }

    fn get_peer_from_torrent(&self, info_hash: &Vec<u8>) -> Vec<Peer> {
        self.peers_by_torrent.get(info_hash).unwrap().to_vec()
    }
}
