use super::connection::ServerConnection;
use super::constants::*;
use super::errors::ServerError;
use super::thread_pool::ThreadPool;
use super::ServerLogger;
use crate::metainfo::Metainfo;
use crate::peer::PeerMessageService;
use log::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
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
    ///  let server: Server = Server::run(client_peer_id, metainfo, 6687);
    ///  
    ///  server.stop().unwrap();
    ///  ```
    ///
    pub fn run(client_peer_id: Vec<u8>, metainfo: Metainfo, port: u16) -> Server {
        let (tx, rx) = mpsc::channel();

        let handle =
            std::thread::spawn(move || Self::listen(LOCALHOST, port, client_peer_id, metainfo, rx));

        Server { sender: tx, handle }
    }

    fn listen(
        address: &str,
        port: u16,
        client_peer_id: Vec<u8>,
        metainfo: Metainfo,
        receiver: Receiver<ServerMessage>,
    ) -> Result<(), ServerError> {
        let (logger, handle) = ServerLogger::new(LOGS_DIR)?;

        let address = format!("{}:{}", address, port);
        let listener: TcpListener = TcpListener::bind(&address)?;
        listener.set_nonblocking(true).map_err(|_| {
            ServerError::ServerCreationError("Couldn't set non blocking mode on server".to_string())
        })?;
        let pool: ThreadPool = ThreadPool::new(POOL_WORKERS)?;
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    info!("Incoming connection");
                    let _ = Server::handle_incoming_connection(
                        stream,
                        metainfo.clone(),
                        client_peer_id.clone(),
                        logger.clone(),
                        &pool,
                    );
                    continue;
                }
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    // This doesen't mean an error ocurred, there just wasn't a connection at the moment
                    if receiver.try_recv().is_ok() {
                        info!("Server received stop message");
                        break;
                    } else {
                        info!("Connection does not exist, sleeping...");
                        thread::sleep(Duration::from_secs(5));
                        continue;
                    }
                }
                Err(err) => return Err(ServerError::TcpStreamError(err)),
            };
        }

        logger.stop();
        handle.join().unwrap();
        Ok(())
    }

    fn handle_incoming_connection(
        stream: TcpStream,
        metainfo: Metainfo,
        client_id: Vec<u8>,
        logger: ServerLogger,
        pool: &ThreadPool,
    ) -> Result<(), ServerError> {
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(120)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;
        let connection_logger = logger;
        pool.execute(|| {
            let message_service = PeerMessageService::from_peer_connection(stream);
            let _ = ServerConnection::new(client_id, metainfo, Box::new(message_service))
                .run(connection_logger, PIECES_DIR);
        });

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
