use crate::application_constants::*;
use crate::application_errors::ApplicationError;
use crate::client::Client;
use crate::config::Config;
use crate::http::HttpsService;
use crate::metainfo::Metainfo;
use crate::peer_connection_manager::new_peer_connection_manager;
use crate::piece_manager::new_piece_manager;
use crate::piece_saver::new_piece_saver;
use crate::server::Server;
use crate::tracker::TrackerResponse;
use crate::tracker::TrackerService;
use crate::ui::{UIMessage, UIMessageSender};
use gtk::{self, glib};
use log::*;
use rand::Rng;

fn init_client(torrent_path: &str) -> Result<Client, ApplicationError> {
    let config = Config::from_path(CONFIG_PATH)?;
    let peer_id = rand::thread_rng().gen::<[u8; 20]>();
    let metainfo = Metainfo::from_torrent(torrent_path)?;
    let http_service = HttpsService::from_url(&metainfo.announce)?;
    let tracker_service = TrackerService::from_metainfo(
        &metainfo,
        config.listen_port,
        &peer_id,
        Box::new(http_service),
    );

    Ok(Client {
        config,
        peer_id,
        metainfo,
        tracker_service,
    })
}

fn init_ui(
    ui_message_sender: Option<glib::Sender<UIMessage>>,
    client: &mut Client,
) -> UIMessageSender {
    let ui_message_sender = match ui_message_sender {
        Some(sender) => UIMessageSender::with_ui(&client.metainfo.info.name, sender),
        None => UIMessageSender::no_ui(),
    };
    ui_message_sender.send_metadata(client.metainfo.clone());
    ui_message_sender
}

fn get_peers_from_tracker(
    client: &mut Client,
    ui_message_sender: UIMessageSender,
) -> Result<TrackerResponse, ApplicationError> {
    let tracker_response = client.tracker_service.get_peers()?;
    ui_message_sender.send_initial_peers(tracker_response.peers.len() as u32);
    Ok(tracker_response)
}

pub fn run_with_torrent(
    torrent_path: &str,
    ui_message_sender: Option<glib::Sender<UIMessage>>,
) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    let mut client: Client = init_client(torrent_path)?;
    let ui_message_sender = init_ui(ui_message_sender, &mut client);

    let tracker_response = get_peers_from_tracker(&mut client, ui_message_sender.clone())?;

    let (server, server_handle) = Server::start(client.peer_id.to_vec(), client.metainfo.clone());
    info!("Server created");

    let (piece_manager_sender, mut piece_manager_worker) = new_piece_manager(
        client.metainfo.info.pieces.len() as u32,
        ui_message_sender.clone(),
    );
    info!("Piece manager created");

    let (piece_saver_sender, piece_saver_worker) = new_piece_saver(
        piece_manager_sender.clone(),
        client.metainfo.info.pieces.clone(),
        client.config.download_path,
    );
    info!("Piece saver created");

    let (peer_connection_manager_sender, mut peer_connection_manager_worker) =
        new_peer_connection_manager(
            piece_manager_sender.clone(),
            piece_saver_sender.clone(),
            &client.metainfo,
            &client.peer_id,
            ui_message_sender,
        );

    info!("Peer connection manager created");
    piece_manager_sender.start(peer_connection_manager_sender.clone());

    let piece_saver_worker_handle = std::thread::spawn(move || {
        piece_saver_worker.listen().unwrap();
    });
    info!("Piece saver worker running");

    let piece_manager_worker_handle = std::thread::spawn(move || {
        let _ = piece_manager_worker.listen(peer_connection_manager_sender);
    });
    info!("Piece manager worker running");

    let peer_connection_manager_worker_handle = std::thread::spawn(move || {
        info!("About to start peer connections");
        peer_connection_manager_worker.start_peer_connections(&tracker_response.peers);
        info!("Peer connections started, about to listen");
        peer_connection_manager_worker.listen().unwrap();
    });
    info!("Peer connection manager worker running");

    info!("Wating download to finish");
    piece_manager_worker_handle.join()?;
    info!("Piece manager stopped running");

    server.stop();
    info!("Server stopped");
    piece_manager_sender.stop();
    info!("Piece manager stopped");
    piece_saver_sender.stop();
    info!("Piece saver stopped");

    trace!("Start closing threads");
    server_handle.join()?;

    piece_saver_worker_handle.join()?;
    peer_connection_manager_worker_handle.join()?;
    info!("Exited Bitorrent client successfully!");
    Ok(())
}
