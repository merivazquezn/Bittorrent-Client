use super::constants::HTTP_PORT;
use super::constants::LOCALHOST;
use super::constants::POOL_WORKERS;
use super::errors::TrackerError;
use bittorrent_rustico::server::ThreadPool;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

enum TrackerMessage {
    Stop,
}

/// Struct that handles the server's acceptor thread.
pub struct TrackerServer {
    sender: Sender<TrackerMessage>,
    handle: thread::JoinHandle<Result<(), TrackerError>>,
}

impl TrackerServer {
    /// Creates a new server that runs a Bittorrent Tracker.
    /// The server starts running and listening inmediatly after created
    ///
    /// # Arguments
    /// * `time_to_sleep` - The time the server sleeps between empty connection reads.
    ///
    /// # Returns
    /// A new Tracker server, of type `TrackerServer`.
    ///
    /// ## Example
    ///
    ///  ```no_run
    ///
    ///  use bittorrent_rustico::server::TrackerServer;
    ///  use std::time::Duration;
    ///
    ///  let tracker_server: TrackerServer = Server::run(Duration::from_secs(1));
    ///  
    ///  server.stop().unwrap();
    ///  ```
    ///
    pub fn new(time_to_sleep: Duration) -> TrackerServer {
        let (tx, rx) = mpsc::channel();
        let listen_port: u16 = HTTP_PORT;
        let ip: &str = LOCALHOST;
        let handle = thread::spawn(move || Self::listen(ip, listen_port, rx, time_to_sleep));
        TrackerServer { sender: tx, handle }
    }

    fn listen(
        ip: &str,
        listen_port: u16,
        receiver: Receiver<TrackerMessage>,
        time_to_sleep: Duration,
    ) -> Result<(), TrackerError> {
        let address: String = format!("{}:{}", ip, listen_port);
        let listener: TcpListener = TcpListener::bind(&address)?;

        listener.set_nonblocking(true).map_err(|_| {
            TrackerError::CreationError("Failed trying to set non-blocking mode".to_string())
        })?;

        let pool: ThreadPool = ThreadPool::new(POOL_WORKERS)?;

        for incoming_stream in listener.incoming() {
            if receiver.try_recv().is_ok() {
                println!("Tracker received stop message");
                break;
            }

            match incoming_stream {
                Ok(stream) => {
                    println!("Tracker: Incoming connection");
                    TrackerServer::handle_incoming_connection(stream, &pool);
                }
                Err(ref err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    // This doesen't mean an error ocurred, there just wasn't an incoming connection at the moment
                    println!(
                        "There aren't any incoming connection at the moment, going to sleep..."
                    );

                    thread::sleep(time_to_sleep);
                }
                Err(err) => return Err(TrackerError::TcpError(err)),
            };
        }

        Ok(())
    }

    fn handle_incoming_connection(_stream: TcpStream, pool: &ThreadPool) {
        pool.execute(move || {
            println!("Handling incoming connection...");
        });
    }

    /// Stops the Tracker.
    /// If the Tracker is in the middle of creating a connection, it may take a little while for it to finish.
    /// # Returns
    ///
    /// ## On succes
    /// `Ok(())`
    ///
    /// ## On error
    /// `Err(TrackerError)`, containing inside the cause of the error
    ///
    /// ## Example
    /// Check the example at the `run` method of the TrackerServer documentation.
    ///
    pub fn stop(self) -> Result<(), TrackerError> {
        let _ = self.sender.send(TrackerMessage::Stop);
        self.handle.join().map_err(|_| TrackerError::JoinError)??;
        Ok(())
    }
}
