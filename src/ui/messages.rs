use crate::metainfo::Metainfo;
use crate::peer::PeerConnectionState;
use gtk::{self, glib};
use log::*;

type TorrentName = String;

pub struct PeerStatistics {
    _peer_id: String,
    _ip: String,
    _port: u16,
    _state: PeerConnectionState,
    _download_rate: u32,
    _upload_rate: u32,
}

pub enum UIMessage {
    AddTorrent(Metainfo),
    TorrentInitialPeers(TorrentName, u32),
    PieceDownloaded(TorrentName),
    NewConnection(TorrentName),
    ClosedConnection(TorrentName),
    PeersStatistics(Vec<PeerStatistics>),
    UpdatePeerStatistics(PeerStatistics),
}

#[derive(Debug, Clone)]
pub struct UIMessageSender {
    pub tx: Option<glib::Sender<UIMessage>>,
    torrent_name: String,
}

impl UIMessageSender {
    pub fn no_ui() -> Self {
        UIMessageSender {
            tx: None,
            torrent_name: "".to_string(),
        }
    }

    pub fn with_ui(torrent_name: &str, tx: glib::Sender<UIMessage>) -> Self {
        UIMessageSender {
            tx: Some(tx),
            torrent_name: torrent_name.to_string(),
        }
    }

    pub fn send_metadata(&self, metainfo: Metainfo) {
        self.send_message_to_ui(UIMessage::AddTorrent(metainfo))
    }

    pub fn send_initial_peers(&self, num_peers: u32) {
        self.send_message_to_ui(UIMessage::TorrentInitialPeers(
            self.torrent_name.clone(),
            num_peers,
        ))
    }

    pub fn send_new_connection(&self) {
        self.send_message_to_ui(UIMessage::NewConnection(self.torrent_name.clone()))
    }

    pub fn send_downloaded_piece(&self) {
        self.send_message_to_ui(UIMessage::PieceDownloaded(self.torrent_name.clone()))
    }

    pub fn send_closed_connection(&self) {
        self.send_message_to_ui(UIMessage::ClosedConnection(self.torrent_name.clone()))
    }

    pub fn send_message_to_ui(&self, message: UIMessage) {
        if let Some(tx) = &self.tx {
            if tx.send(message).is_err() {
                error!("Failed to send message to UI");
            }
        }
    }
}
