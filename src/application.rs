use crate::application_constants::*;
use crate::application_errors::ApplicationError;
use crate::config::Config;
use crate::http::HttpsService;
use crate::metainfo::Metainfo;
use crate::peer::PeerConnection;
use crate::peer::PeerMessageService;
use crate::peer_connection_manager::new_peer_connection_manager;
use crate::piece_manager::new_piece_manager;
use crate::piece_saver::new_piece_saver;
use crate::server::Server;

use crate::tracker::TrackerService;
use crate::ui::{UIMessage, UIMessageSender};
use gtk::{self, glib};
use log::*;
use rand::Rng;

pub fn run_with_torrent(
    torrent_path: &str,
    ui_message_sender: Option<glib::Sender<UIMessage>>,
) -> Result<(), ApplicationError> {
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
    let ui_message_sender = match ui_message_sender {
        Some(sender) => UIMessageSender::with_ui(&metainfo.info.name, sender),
        None => UIMessageSender::no_ui(),
    };
    ui_message_sender.send_metadata(metainfo.clone());
    let http_service = HttpsService::from_url(&metainfo.announce)?;
    let mut tracker_service = TrackerService::from_metainfo(
        &metainfo,
        config.listen_port,
        &client_peer_id,
        Box::new(http_service),
    );
    info!("Fetching peers from tracker");
    let tracker_response = tracker_service.get_peers()?;
    ui_message_sender.send_initial_peers(tracker_response.peers.len() as u32);
    info!("Fetched peers from Tracker successfully");

    let (server, server_handle) = Server::start(client_peer_id.to_vec(), metainfo.clone());

    let (piece_manager_sender, mut piece_manager_worker) =
        new_piece_manager(metainfo.info.pieces.len() as u32, ui_message_sender.clone());

    let (piece_saver_sender, piece_saver_worker) = new_piece_saver(
        piece_manager_sender.clone(),
        metainfo.info.pieces.clone(),
        config.download_path,
    );
    let (peer_connection_manager_sender, peer_connection_manager_worker) =
        new_peer_connection_manager(
            piece_manager_sender.clone(),
            piece_saver_sender.clone(),
            &metainfo,
            &client_peer_id,
            ui_message_sender.clone(),
        );
    piece_manager_sender.start(peer_connection_manager_sender.clone());
    let piece_saver_worker_handle = std::thread::spawn(move || {
        piece_saver_worker.listen().unwrap();
    });
    let piece_manager_worker_handle = std::thread::spawn(move || {
        let _ = piece_manager_worker.listen(peer_connection_manager_sender);
    });

    let peer_connection_manager_worker_handle = std::thread::spawn(move || {
        peer_connection_manager_worker.listen().unwrap();
    });
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
            ui_message_sender,
        )
        .run()?;
        info!("Finished download of piece {} from peer: {}", 0, peer.ip);
    }

    trace!("Start closing threads");

    server.stop();
    piece_manager_sender.stop();
    piece_saver_sender.stop();

    server_handle.join()?;
    piece_manager_worker_handle.join()?;
    piece_saver_worker_handle.join()?;
    peer_connection_manager_worker_handle.join()?;
    info!("Exited Bitorrent client successfully!");
    Ok(())
}
