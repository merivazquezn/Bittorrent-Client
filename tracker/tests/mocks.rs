use bittorrent_rustico::download_manager;
use std::collections::HashMap;
use std::io::Write;
use tracker::http::{HttpError, HttpGetRequest, IHttpService, IHttpServiceFactory};

use std::thread;
use tracker::aggregator::Aggregator;
use tracker::metrics::new_metrics;
use tracker::server::TrackerServer;

#[derive(Clone)]
pub struct MockHttpService {
    pub path: String,
    pub params: HashMap<String, String>,
    pub results_path: String,
    pub client_address: std::net::SocketAddr,
}

impl IHttpService for MockHttpService {
    fn parse_request(&mut self) -> Result<tracker::http::HttpGetRequest, tracker::http::HttpError> {
        Ok(HttpGetRequest {
            params: self.params.clone(),
            path: self.path.clone(),
        })
    }

    fn send_ok_response(
        &mut self,
        content: Vec<u8>,
        content_type: String,
    ) -> Result<(), tracker::http::HttpError> {
        download_manager::create_directory(&format!(
            "{}/tests/{}",
            std::env::current_dir().unwrap().to_str().unwrap(),
            self.results_path.clone()
        ))
        .unwrap();

        let path = format!(
            "{}/tests/{}/{}",
            std::env::current_dir().unwrap().to_str().unwrap(),
            self.results_path.clone(),
            "content"
        );
        let mut file = std::fs::File::create(path)?;
        file.write_all(&content)?;
        Ok(())
    }

    fn send_not_found(&mut self) -> Result<(), tracker::http::HttpError> {
        Ok(())
    }

    fn get_client_address(&self) -> std::net::SocketAddr {
        self.client_address
    }
}

pub struct MockHttpServiceFactory {
    pub connections: Vec<MockHttpService>,
    pub current_connection: usize,
    pub factory_sender: std::sync::mpsc::Sender<()>,
}

impl IHttpServiceFactory for MockHttpServiceFactory {
    fn get_new_connection(&mut self) -> Result<Box<dyn IHttpService>, HttpError> {
        if self.current_connection >= self.connections.len() {
            self.factory_sender.send(()).unwrap();
            return Err(HttpError::HttpError(
                "Could not accept connection".to_string(),
            ));
        }
        let connection = self.connections[self.current_connection].clone();
        self.current_connection += 1;
        Ok(Box::new(connection))
    }
}

pub fn create_mock_connection(
    left: usize,
    uploaded: usize,
    downloaded: usize,
    info_hash: &str,
    peer_id: &str,
    request_number: usize,
    client_address: &str,
) -> MockHttpService {
    let mut params = HashMap::new();
    params.insert("left".to_string(), left.to_string());
    params.insert("uploaded".to_string(), uploaded.to_string());
    params.insert("downloaded".to_string(), downloaded.to_string());
    params.insert("info_hash".to_string(), info_hash.to_string());
    params.insert("peer_id".to_string(), peer_id.to_string());
    //creat socket addr (ipv4 version) from client_address

    MockHttpService {
        path: "announce".to_string(),
        params,
        results_path: format!("{}/{}", "announce", request_number),
        client_address: client_address.parse().unwrap(),
    }
}

pub fn run_mock_server(peer_connections: Vec<MockHttpService>) {
    let (factory_sender, factory_receiver) = std::sync::mpsc::channel();
    let connections_factory: Box<dyn IHttpServiceFactory + Send> =
        Box::new(MockHttpServiceFactory {
            connections: peer_connections,
            current_connection: 0,
            factory_sender,
        });

    let main_handle = thread::spawn(move || {
        let (metrics_sender, mut metrics_worker) = new_metrics(1);

        let aggregator: Aggregator = match Aggregator::start() {
            Ok(aggregator) => aggregator,
            Err(_) => {
                panic!("error creating aggregator");
            }
        };

        let mut aggregator_worker = aggregator.worker;
        let _ = thread::spawn(move || {
            let _ = metrics_worker.listen();
        });

        let metrics = metrics_sender.clone();
        let _ = thread::spawn(move || {
            let _ = aggregator_worker.listen(metrics);
        });

        let (tracker_sender, tracker_receiver) = std::sync::mpsc::channel();

        let handle_tracker = thread::spawn(move || {
            TrackerServer::listen(
                connections_factory,
                aggregator.sender,
                metrics_sender,
                1,
                tracker_receiver,
            )
            .unwrap()
        });

        factory_receiver.recv().unwrap();
        tracker_sender.send(()).unwrap();
        handle_tracker.join().unwrap()
    });
    main_handle.join().unwrap();
}
