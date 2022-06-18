use super::connection::ServerConnection;
use super::constants::*;
use super::errors::ServerError;
use super::thread_pool::ThreadPool;
use crate::metainfo::Metainfo;
use crate::peer::PeerMessageService;
use std::net::TcpListener;
use std::thread::JoinHandle;

#[allow(dead_code)]
pub struct Server;

impl Server {
    pub fn start(client_peer_id: Vec<u8>, metainfo: Metainfo) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let _ = Self::listen(LOCALHOST, client_peer_id, metainfo);
        })
    }

    fn listen(
        address: &str,
        client_peer_id: Vec<u8>,
        metainfo: Metainfo,
    ) -> Result<(), ServerError> {
        let listener: TcpListener = TcpListener::bind(address)?;
        let pool = ThreadPool::new(POOL_WORKERS)?;
        for stream in listener.incoming() {
            let stream = stream?;
            let metainfo = metainfo.clone();
            let client_peer_id = client_peer_id.clone();
            pool.execute(|| {
                let message_service = PeerMessageService::from_peer_connection(stream);
                ServerConnection::new(client_peer_id, metainfo, Box::new(message_service)).run();
            });
        }

        Ok(())
    }
}
