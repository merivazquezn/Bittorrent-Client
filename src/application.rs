use crate::application_constants::*;
use crate::application_errors::ApplicationError;
use crate::config::Config;
use crate::http::HttpsService;
use crate::metainfo::Metainfo;
use crate::peer::PeerConnection;
use crate::peer::PeerMessageService;
use crate::peer_connection_manager::PeerConnectionManager;
use crate::piece_manager::PieceManager;
use crate::piece_saver::PieceSaver;
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
    let http_service = HttpsService::from_url(&metainfo.announce)?;
    let mut tracker_service = TrackerService::from_metainfo(
        &metainfo,
        config.listen_port,
        &client_peer_id,
        Box::new(http_service),
    );
    info!("Fetching peers from tracker");
    let tracker_response = tracker_service.get_peers()?;
    info!("Fetched peers from Tracker successfully");

    /* *********************************************************************** */

    let (piece_manager, piece_manager_handle) = PieceManager::new();

    let (peer_connection_manager, peer_connection_manager_handle) = PeerConnectionManager::new();

    let (piece_saver, piece_saver_handle) = PieceSaver::new(piece_manager.clone());

    piece_manager.start(peer_connection_manager.clone());
    peer_connection_manager.start(piece_manager.clone(), piece_saver.clone());

    if let Some(peer) = tracker_response.peers.get(0) {
        info!(
            "Trying to connect to peer {} and download piece {}",
            peer.ip, 0
        );
        let peer_message_stream = PeerMessageService::connect_to_peer(peer)?;
        PeerConnection::new(
            peer,
            &client_peer_id,
            &metainfo,
            Box::new(peer_message_stream),
        )
        .run()?;
        info!("Finished download of piece {} from peer: {}", 0, peer.ip);
    }

    trace!("Start closing threads");

    piece_manager.stop();
    peer_connection_manager.stop();
    piece_saver.stop();

    piece_manager_handle.join()?;
    peer_connection_manager_handle.join()?;
    piece_saver_handle.join()?;

    info!("Exited Bitorrent client successfully!");
    Ok(())
}
