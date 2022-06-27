use super::connection::ServerConnection;
use super::constants::*;
use super::errors::ServerError;
use super::thread_pool::ThreadPool;
use super::ServerLogger;
use crate::application_errors::ApplicationError;
use crate::metainfo::Metainfo;
use crate::peer::PeerMessageService;
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

pub enum ServerMessage {
    Stop,
}

pub struct Server {
    sender: Sender<ServerMessage>,
    handle: JoinHandle<()>,
}

impl Server {
    pub fn run(client_peer_id: Vec<u8>, metainfo: Metainfo) -> Server {
        let (tx, rx) = mpsc::channel();

        let handle = std::thread::spawn(move || {
            let _ = Self::listen(LOCALHOST, client_peer_id, metainfo, rx);
        });

        Server { sender: tx, handle }
    }

    fn listen(
        address: &str,
        client_peer_id: Vec<u8>,
        metainfo: Metainfo,
        receiver: Receiver<ServerMessage>,
    ) -> Result<(), ServerError> {
        let (logger, handle) = ServerLogger::new(LOGS_DIR)?;

        let listener: TcpListener = TcpListener::bind(address)?;
        let pool: ThreadPool = ThreadPool::new(POOL_WORKERS)?;
        for stream in listener.incoming() {
            if receiver.try_recv().is_ok() {
                break;
            }

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

    pub fn stop(self) -> Result<(), ApplicationError> {
        let _ = self.sender.send(ServerMessage::Stop);
        self.handle.join()?;
        Ok(())
    }
}
