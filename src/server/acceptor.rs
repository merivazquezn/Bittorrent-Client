use super::connection::ServerConnection;
use super::constants::*;
use super::errors::ServerError;
use super::thread_pool::ThreadPool;
use super::ServerLogger;
use crate::metainfo::Metainfo;
use crate::peer::PeerMessageService;
use crate::tracker::Event;
use crate::tracker::ITrackerService;
use crate::tracker::TrackerService;
use log::*;
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

const TRACKER_INTERVAL_IN_SECONDS: u64 = 20;

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
    ///  ```no_compile
    ///
    ///  use bittorrent_rustico::server::Server;
    ///  use bittorrent_rustico::metainfo::Metainfo;   
    ///  use rand::Rng;
    ///  use std::time::Duration;
    ///
    ///  let metainfo = Metainfo::from_torrent("debian.torrent").unwrap();
    ///  let client_peer_id = rand::thread_rng().gen::<[u8; 20]>().to_vec();
    ///
    ///  let server: Server = Server::run(client_peer_id, metainfo, 6687, Duration::from_secs(10), "./downloads/pieces");
    ///  
    ///  server.stop().unwrap();
    ///  ```
    ///
    pub fn run(
        client_peer_id: Vec<u8>,
        metainfo: Metainfo,
        port: u16,
        time_to_sleep: Duration,
        pieces_dir: &str,
        tracker_service: TrackerService,
    ) -> Server {
        let (tx, rx) = mpsc::channel();
        let pieces_dir_clone = String::from(pieces_dir);
        let address: SocketAddr = socket_from_address(LOCALHOST.to_string(), port);

        let handle = std::thread::spawn(move || {
            Self::listen(
                address,
                client_peer_id,
                metainfo,
                rx,
                time_to_sleep,
                &pieces_dir_clone,
                tracker_service,
            )
        });

        Server { sender: tx, handle }
    }

    fn listen(
        address: SocketAddr,
        client_peer_id: Vec<u8>,
        metainfo: Metainfo,
        receiver: Receiver<ServerMessage>,
        time_to_sleep: Duration,
        pieces_dir: &str,
        mut tracker_service: TrackerService,
    ) -> Result<(), ServerError> {
        let (logger, handle) = ServerLogger::new(LOGS_DIR)?;
        let address = format!("{}:{}", address.ip(), address.port());
        let mut last_announce = std::time::Instant::now();
        let listener: TcpListener = TcpListener::bind(&address)?;
        listener.set_nonblocking(true).map_err(|_| {
            ServerError::ServerCreationError("Couldn't set non blocking mode on server".to_string())
        })?;
        let pool: ThreadPool = ThreadPool::new(25)?;
        for stream in listener.incoming() {
            if receiver.try_recv().is_ok() {
                info!("Server received stop message");
                break;
            }

            match stream {
                Ok(stream) => {
                    info!("Server: Incoming connection");
                    println!(
                        "handle incomming connection return data:{:?}",
                        Server::handle_incoming_connection(
                            stream,
                            metainfo.clone(),
                            client_peer_id.clone(),
                            logger.clone(),
                            &pool,
                            pieces_dir,
                        )
                    );
                }
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    // This doesen't mean an error ocurred, there just wasn't a connection at the moment
                    if last_announce.elapsed().as_secs() > TRACKER_INTERVAL_IN_SECONDS {
                        println!("announcing");
                        let _ = tracker_service.announce(None);
                        last_announce = std::time::Instant::now();
                    }

                    thread::sleep(time_to_sleep);
                }
                Err(err) => return Err(ServerError::TcpStreamError(err)),
            };
        }

        logger.stop();
        handle.join().unwrap();

        let _ = tracker_service.announce(Some(Event::Stopped));
        Ok(())
    }

    fn handle_incoming_connection(
        stream: TcpStream,
        metainfo: Metainfo,
        client_id: Vec<u8>,
        logger: ServerLogger,
        pool: &ThreadPool,
        pieces_dir: &str,
    ) -> Result<(), ServerError> {
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(100)))?;
        stream.set_write_timeout(Some(Duration::from_secs(100)))?;
        let connection_logger = logger;
        let dir_clone = String::from(pieces_dir);
        pool.execute(move || {
            info!("inside pool execution");
            let message_service = PeerMessageService::from_peer_connection(stream);
            let _ = ServerConnection::new(client_id, metainfo, Box::new(message_service))
                .run(connection_logger, &dir_clone);
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

fn socket_from_address(ip: String, port: u16) -> SocketAddr {
    let ip: IpAddr = ip.parse::<IpAddr>().unwrap();
    SocketAddr::new(ip, port)
}
