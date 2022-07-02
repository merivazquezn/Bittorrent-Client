use crate::application_errors::ApplicationError;
use crate::client::{ClientInfo, TorrentClient};
use crate::tracker::get_response_from_tracker;
use crate::ui::{init_ui, UIMessage};
use gtk::{self, glib};
use log::*;

pub fn run_with_torrent(
    torrent_path: &str,
    ui_message_sender: Option<glib::Sender<UIMessage>>,
) -> Result<(), ApplicationError> {
    let mut client_info = ClientInfo::new(torrent_path)?;
    let ui_message_sender = init_ui(ui_message_sender, &mut client_info);

    // let server = Server::run(client_info.peer_id.to_vec(), client_info.metainfo.clone());
    let (tracker_response, tracker_service) =
        get_response_from_tracker(&mut client_info, ui_message_sender.clone())?;

    let client: TorrentClient = TorrentClient::new(&client_info, ui_message_sender)?;
    client.run(client_info, Box::new(tracker_service), tracker_response)?;

    // server.stop()?;

    info!("Exited bittorrent client succesfully!");
    Ok(())
}
