use super::constants::HTTP_PORT;
use super::constants::LOCALHOST;
use super::constants::POOL_WORKERS;
use super::errors::TrackerError;
use bittorrent_rustico::logger::CustomLogger;
use bittorrent_rustico::server::ThreadPool;
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

const LOGGER: CustomLogger = CustomLogger::init("tracker");
enum TrackerMessage {
    Stop,
}

/// Struct that handles the server's acceptor thread.
pub struct TrackerServer {
    sender: Sender<TrackerMessage>,
    pub handle: thread::JoinHandle<Result<(), TrackerError>>,
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
    pub fn new(testTx: Sender<TcpStream>) -> TrackerServer {
        let (tx, rx) = mpsc::channel();
        let listen_port: u16 = HTTP_PORT;
        let ip: &str = LOCALHOST;
        let handle = thread::spawn(move || Self::listen(ip, listen_port, testTx));
        TrackerServer { sender: tx, handle }
    }

    fn listen(ip: &str, listen_port: u16, testTx: Sender<TcpStream>) -> Result<(), TrackerError> {
        let address: String = format!("{}:{}", ip, listen_port);
        let listener: TcpListener = TcpListener::bind(&address)?;

        LOGGER.info(format!("Listening on {}", address));
        let pool: ThreadPool = ThreadPool::new(POOL_WORKERS)?;

        for incoming_stream in listener.incoming() {
            match incoming_stream {
                Ok(stream) => {
                    let stream_clone = stream.try_clone()?;
                    LOGGER.info(format!("Tracker: Incoming connection"));
                    TrackerServer::handle_incoming_connection(stream, &pool);
                }
                Err(err) => {
                    LOGGER.error(format!("Tracker: Error: {}", err));
                    return Err(TrackerError::TcpError(err));
                }
            };
        }

        Ok(())
    }

    fn handle_incoming_connection(mut stream: TcpStream, pool: &ThreadPool) {
        pool.execute(move || {
            let mut buffer = [0; 2048];
            stream.read(&mut buffer).unwrap();

            // buffer is the HTTP request, read the path and extract the extension of the file requested
            let get = b"GET";
            let (status_line, filename) = if buffer.starts_with(get) {
                // path is the part of the request after the GET and before the HTTP/1.1
                let request = std::str::from_utf8(&buffer).unwrap();
                let path = request.trim_start_matches("GET /");
                let mut path = path.split(" ").next().unwrap();

                if path == "stats" || path.is_empty() {
                    path = "index.html";
                }

                LOGGER.info(format!("path: {}", path));
                ("HTTP/1.1 200 OK", path)
            } else {
                ("HTTP/1.1 404 NOT FOUND", "404.html")
            };

            LOGGER.info(format!("{}", filename));
            let contents = fs::read(format!("{}{}", "frontend/build/", filename)).unwrap();

            // based on the extension, write the correct content type to the header
            let content_type = if filename.ends_with(".html") {
                "text/html"
            } else if filename.ends_with(".css") {
                "text/css"
            } else if filename.ends_with(".js") {
                "application/javascript"
            } else if filename.ends_with(".png") {
                "image/png"
            } else if filename.ends_with(".jpg") {
                "image/jpeg"
            } else if filename.ends_with(".gif") {
                "image/gif"
            } else if filename.ends_with(".svg") {
                "image/svg+xml"
            } else if filename.ends_with(".ico") {
                "image/x-icon"
            } else {
                "text/plain"
            };
            let response = format!(
                "{}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n",
                status_line,
                contents.len(),
                content_type
            );
            let response = response.as_bytes();
            let response = response
                .iter()
                .chain(contents.iter())
                .cloned()
                .collect::<Vec<_>>();

            stream.write(&response).unwrap();
            stream.flush().unwrap();
        })
    }
}
