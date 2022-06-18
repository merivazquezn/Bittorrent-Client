use crate::metainfo::Metainfo;
use crate::peer::IServerPeerMessageService;
use log::*;

#[allow(dead_code)]
pub struct ServerConnection {
    message_service: Box<dyn IServerPeerMessageService>,
    metainfo: Metainfo,
    client_peer_id: Vec<u8>,
}

impl ServerConnection {
    pub fn new(
        client_peer_id: Vec<u8>,
        metainfo: Metainfo,
        message_service: Box<dyn IServerPeerMessageService>,
    ) -> Self {
        Self {
            client_peer_id: client_peer_id.to_vec(),
            metainfo,
            message_service,
        }
    }

    pub fn run(&mut self) {
        self.message_service
            .handshake(&self.metainfo.info_hash, &self.client_peer_id)
            .unwrap();

        loop {
            let message = self.message_service.wait_for_message().unwrap();

            // manejar en match lo que el server deba manejar, para que funcione :D
            trace!("{:?}", message);
        }
    }
}
