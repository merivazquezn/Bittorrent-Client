//! Our GObject subclass for carrying a name and count for the ListBox model
//!
//! Both name and count are stored in a RefCell to allow for interior mutability
//! and are exposed via normal GObject properties. This allows us to use property
//! bindings below to bind the values with what widgets display in the UI

mod imp;

use gtk::glib;
use sha1::{Digest, Sha1};

use crate::peer::PeerState;

// Public part of the TorrentInformation type. This behaves like a normal gtk-rs-style GObject
// binding
glib::wrapper! {
    pub struct DownloadStatistics(ObjectSubclass<imp::DownloadStatistics>);
}

// Constructor for new instances. This simply calls glib::Object::new() with
// initial values for our two properties and then returns the new instance
impl DownloadStatistics {
    pub fn new(
        torrentname: &str,
        id: &[u8],
        ip: &str,
        port: u16,
        clientstate: PeerState,
        peerstate: PeerState,
    ) -> DownloadStatistics {
        let client_interested = match clientstate.interested {
            true => "interested",
            false => "not interested",
        };
        let client_choked = match clientstate.chocked {
            true => "choked",
            false => "not choked",
        };

        let peer_interested = match peerstate.interested {
            true => "interested",
            false => "not interested",
        };
        let peer_choked = match peerstate.chocked {
            true => "choked",
            false => "not choked",
        };

        let client_state = client_interested.to_string() + " and " + client_choked;
        let peer_state = peer_interested.to_string() + " and " + peer_choked;
        let ipport = format!("{}:{}", ip, port);
        glib::Object::new(&[
            ("torrentname", &torrentname),
            ("id", &Self::sha1_of(id)),
            ("ipport", &ipport),
            ("clientstate", &client_state),
            ("peerstate", &peer_state),
        ])
        .expect("Failed to create row data")
    }

    // function that converts Vec<u8> bytes to ascii characters
    pub fn bytes_to_ascii(bytes: &[u8]) -> String {
        format!("{:02x?}", bytes)
            .replace('[', "")
            .replace(']', "")
            .replace(", ", "")
    }

    pub fn sha1_of(vec: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(vec);
        Self::bytes_to_ascii(&hasher.finalize())
    }
}
