use crate::metainfo::Metainfo;
use crate::peer::PeerConnectionState;
use gtk::{self, glib};
use log::*;

type TorrentName = String;

#[derive(Clone)]
pub struct PeerStatistics {
    pub torrentname: String,
    pub peerid: Vec<u8>,
    pub ip: String,
    pub port: u16,
    pub state: PeerConnectionState,
    pub downloadrate: u32,
    pub uploadrate: u32,
}

pub enum UIMessage {
    AddTorrent(Metainfo),
    TorrentInitialPeers(TorrentName, u32),
    PieceDownloaded(TorrentName, Vec<u8>),
    NewConnection(TorrentName),
    ClosedConnection(TorrentName, Vec<u8>),
    AddPeerStatistics(PeerStatistics),
    UpdatePeerUploadRate(f32, Vec<u8>),
    UpdatePeerDownloadRate(f32, Vec<u8>),
    UpdateDownloadedPiece(Vec<u8>),
    UpdatePeerConnectionState(Vec<u8>, PeerConnectionState),
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

    pub fn send_downloaded_piece(&self, peer_id: Vec<u8>) {
        self.send_message_to_ui(UIMessage::PieceDownloaded(
            self.torrent_name.clone(),
            peer_id,
        ))
    }

    pub fn send_closed_connection(&self, peer_id: Vec<u8>) {
        self.send_message_to_ui(UIMessage::ClosedConnection(
            self.torrent_name.clone(),
            peer_id,
        ))
    }

    pub fn send_peer_statistics(&self, peer_statistics: PeerStatistics) {
        self.send_message_to_ui(UIMessage::AddPeerStatistics(peer_statistics))
    }

    pub fn update_peer_state(&self, peer_id: Vec<u8>, state: PeerConnectionState) {
        self.send_message_to_ui(UIMessage::UpdatePeerConnectionState(peer_id, state))
    }

    pub fn send_upload_rate(&self, rate: f32, peer_id: &[u8]) {
        self.send_message_to_ui(UIMessage::UpdatePeerUploadRate(rate, peer_id.to_vec()))
    }
    pub fn send_download_rate(&self, rate: f32, peer_id: &[u8]) {
        self.send_message_to_ui(UIMessage::UpdatePeerDownloadRate(rate, peer_id.to_vec()))
    }

    pub fn send_message_to_ui(&self, message: UIMessage) {
        if let Some(tx) = &self.tx {
            if tx.send(message).is_err() {
                error!("Failed to send message to UI");
            }
        }
    }
}
