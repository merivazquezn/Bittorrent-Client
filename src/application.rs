use crate::application_errors::ApplicationError;
use crate::client::{ClientInfo, TorrentClient};
use crate::constants::TIME_BETWEEN_ACCEPTS;
use crate::download_manager::get_existing_pieces;
use crate::server::Server;
use crate::server::PIECES_DIR;
use crate::tracker::get_response_from_tracker;
use crate::ui::{init_ui, UIMessage};
use gtk::{self, glib};
use log::*;

pub fn run_with_torrent(
    torrent_path: &str,
    config_path: &str,
    ui_message_sender: Option<glib::Sender<UIMessage>>,
) -> Result<(), ApplicationError> {
    let mut client_info = ClientInfo::new(torrent_path, config_path)?;
    let ui_message_sender = init_ui(ui_message_sender, &mut client_info);

    let pieces_dir = format!(
        "{}/{}/pieces",
        client_info.config.download_path, client_info.metainfo.info.name
    );

    let server = Server::run(
        client_info.peer_id.to_vec(),
        client_info.metainfo.clone(),
        client_info.config.listen_port,
        TIME_BETWEEN_ACCEPTS,
        &pieces_dir,
    );
    let initial_pieces: Vec<u32> =
        get_existing_pieces(client_info.metainfo.get_piece_count(), pieces_dir.as_str());
    println!("{}/pieces", client_info.config.download_path);
    println!("i've got pieces: {:?}", initial_pieces);

    for _ in initial_pieces.clone() {
        ui_message_sender.send_downloaded_piece(client_info.peer_id.to_vec());
    }

    let (tracker_response, tracker_service) = get_response_from_tracker(
        &mut client_info,
        ui_message_sender.clone(),
        initial_pieces.clone(),
    )?;

    let client: TorrentClient =
        TorrentClient::new(&client_info, ui_message_sender, initial_pieces)?;
    client.run(client_info, Box::new(tracker_service), tracker_response)?;

    //server.stop()?;

    info!("Exited bittorrent client succesfully!");
    Ok(())
}
