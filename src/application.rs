use log::*;
use rand::Rng;
// use all the modules config, peer, tracker, metainfo
use crate::application_constants::*;
use crate::application_errors::ApplicationError;
use crate::config::Config;
use crate::http::HttpsConnection;
use crate::logger::Logger;
use crate::metainfo::Metainfo;
use crate::peer::{PeerConnection, PeerMessageStream};
use crate::tracker::TrackerService;
use std::thread;

pub fn run_with_torrent(torrent_path: &str) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    info!("Starting bittorrent client...");
    let client_peer_id = rand::thread_rng().gen::<[u8; 20]>();
    let config = Config::from_path(CONFIG_PATH)?;
    let metainfo = Metainfo::from_torrent(torrent_path)?;
    let http_service = HttpsConnection::from_url(&metainfo.announce)?;
    let mut tracker_service = TrackerService::from_metainfo(
        &metainfo,
        config.listen_port,
        &client_peer_id,
        Box::new(http_service),
    );
    let tracker_response = tracker_service.get_peers()?;

    info!("Received peers from tracker succesfully");

    // Starts Logger
    let (logger, mut logger_listener) = Logger::new(LOG_DIR)?;
    thread::spawn(move || {
        println!("Logger starts listening...");
        logger_listener.listen();
    });

    if let Some(peer) = tracker_response.peers.get(0) {
        let peer_message_stream = PeerMessageStream::connect_to_peer(peer).unwrap();
        PeerConnection::new(
            peer,
            &client_peer_id,
            &metainfo,
            Box::new(peer_message_stream),
        )
        .run()
        .unwrap();
    }

    info!("Starts logging pieces");
    logger.log_piece(12);
    logger.log_piece(14);
    logger.log_piece(65);

    info!("Exited Bitorrent client successfully!");
    Ok(())
}
