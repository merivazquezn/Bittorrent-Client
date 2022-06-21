use super::connection::ServerConnection;
use super::constants::*;
use super::errors::ServerError;
use super::thread_pool::ThreadPool;
use super::ServerLogger;
use crate::metainfo::Metainfo;
use crate::peer::PeerMessageService;
use std::net::TcpListener;
use std::thread::JoinHandle;
use std::time::Duration;

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
        let (logger, handle) = ServerLogger::new(LOGS_DIR)?;

        let listener: TcpListener = TcpListener::bind(address)?;
        let pool = ThreadPool::new(POOL_WORKERS)?;
        for stream in listener.incoming() {
            let stream = stream?;
            stream.set_read_timeout(Some(Duration::new(SERVER_READ_TIMEOUT, 0)))?;
            stream.set_write_timeout(Some(Duration::new(SERVER_WRITE_TIMEOUT, 0)))?;

            let metainfo = metainfo.clone();
            let client_peer_id = client_peer_id.clone();
            let connection_logger = logger.clone();
            pool.execute(|| {
                let message_service = PeerMessageService::from_peer_connection(stream);
                let _ = ServerConnection::new(client_peer_id, metainfo, Box::new(message_service))
                    .run(connection_logger, PIECES_DIR);
            });
        }

        logger.stop();
        handle.join().unwrap();
        Ok(())
    }
}
