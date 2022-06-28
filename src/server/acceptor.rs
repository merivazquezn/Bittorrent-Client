use super::connection::ServerConnection;
use super::constants::*;
use super::errors::ServerError;
use super::thread_pool::ThreadPool;
use super::ServerLogger;
use crate::metainfo::Metainfo;
use crate::peer::PeerMessageService;
use std::net::TcpListener;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

enum ServerMessage {
    Stop,
}

/// Struct that handles the server's acceptor thread.
/// It accepts connections and spawns a thread for each connection.
pub struct Server {
    sender: Sender<ServerMessage>,
    handle: JoinHandle<Result<(), ServerError>>,
}

impl Server {
    /// Creates a new server.
    /// The server starts running and listening inmediatly after created
    ///
    /// # Arguments
    /// * `metainfo` - The metainfo struct of the torrent file.
    /// * `client_peer_id` - The peer_id the client generated in order to identify itself.
    ///
    /// # Returns
    /// A new server, of type `Server`.
    ///
    /// ## Example
    ///
    ///  ```no_run
    ///
    ///  use bittorrent_rustico::server::Server;
    ///  use bittorrent_rustico::metainfo::Metainfo;   
    ///  use rand::Rng;
    ///
    ///  let metainfo = Metainfo::from_torrent("debian.torrent").unwrap();
    ///  let client_peer_id = rand::thread_rng().gen::<[u8; 20]>().to_vec();
    ///
    ///  let server: Server = Server::run(client_peer_id, metainfo);
    ///  
    ///  server.stop().unwrap();
    ///  ```
    ///
    pub fn run(client_peer_id: Vec<u8>, metainfo: Metainfo) -> Server {
        let (tx, rx) = mpsc::channel();

        let handle =
            std::thread::spawn(move || Self::listen(LOCALHOST, client_peer_id, metainfo, rx));

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

    /// Stops the server.
    /// If the server is in the middle of creating a connection, it may take a little while for it to finish.
    /// # Returns
    ///
    /// ## On succes
    /// `Ok(())`
    ///
    /// ## On error
    /// `Err(ServerError)`, containing inside the cause of the error
    ///
    /// ## Example
    /// Check the example at the `run` method of the Server
    ///
    pub fn stop(self) -> Result<(), ServerError> {
        let _ = self.sender.send(ServerMessage::Stop);
        self.handle.join().map_err(|_| ServerError::JoinError)??;

        Ok(())
    }
}
