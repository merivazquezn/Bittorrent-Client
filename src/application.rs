use crate::application_errors::ApplicationError;
use crate::client::{ClientInfo, TorrentClient};
use crate::peer::Peer;
use crate::server::Server;
use crate::tracker::get_peers_from_tracker;
use crate::ui::{init_ui, UIMessage};
use gtk::{self, glib};
use log::*;

pub fn run_with_torrent(
    torrent_path: &str,
    ui_message_sender: Option<glib::Sender<UIMessage>>,
) -> Result<(), ApplicationError> {
    pretty_env_logger::init();
    let mut client_info = ClientInfo::new(torrent_path)?;
    let ui_message_sender = init_ui(ui_message_sender, &mut client_info);

    let server = Server::run(client_info.peer_id.to_vec(), client_info.metainfo.clone());
    let peers: Vec<Peer> = get_peers_from_tracker(&mut client_info, ui_message_sender.clone())?;

    let client: TorrentClient = TorrentClient::new(&client_info, ui_message_sender)?;
    client.run_with_peers(peers, client_info)?;

    server.stop()?;

    info!("Exited bittorrent client succesfully!");
    Ok(())
}
