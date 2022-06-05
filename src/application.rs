use crate::application_constants::*;
use crate::application_errors::ApplicationError;
use crate::config::Config;
use crate::http::HttpsConnection;
use crate::metainfo::Metainfo;
use crate::peer::{PeerConnection, PeerMessageStream};
use crate::tracker::TrackerService;
use log::*;
use rand::Rng;

pub fn run_with_torrent(torrent_path: &str) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    info!("Starting bittorrent client...");
    let client_peer_id = rand::thread_rng().gen::<[u8; 20]>();
    let config = Config::from_path(CONFIG_PATH)?;
    info!("Read client configuration successfully");
    let metainfo = Metainfo::from_torrent(torrent_path)?;
    info!(
        "Parsed Metainfo (torrent file) successfully. I'll try to download {}",
        metainfo.info.name
    );
    let http_service = HttpsConnection::from_url(&metainfo.announce)?;
    let mut tracker_service = TrackerService::from_metainfo(
        &metainfo,
        config.listen_port,
        &client_peer_id,
        Box::new(http_service),
    );
    info!("Fetching peers from tracker");
    let tracker_response = tracker_service.get_peers()?;
    info!("Fetched peers from Tracker successfully");
    if let Some(peer) = tracker_response.peers.get(0) {
        info!(
            "Trying to connect to peer {} and download piece {}",
            peer.ip, 0
        );
        let peer_message_stream = PeerMessageStream::connect_to_peer(peer)?;
        PeerConnection::new(
            peer,
            &client_peer_id,
            &metainfo,
            Box::new(peer_message_stream),
        )
        .run()?;
        info!("Finished download of piece {} from peer: {}", 0, peer.ip);
    }
    info!("Exited Bitorrent client successfully!");
    Ok(())
}
