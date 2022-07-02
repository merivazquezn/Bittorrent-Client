//! Defines our custom model

mod imp;

use super::download_statistics_row::DownloadStatistics;
use crate::peer::PeerConnectionState;
use glib::subclass::prelude::*;
use gtk::{gio, glib, prelude::*};

// Public part of the Model type.
glib::wrapper! {
    pub struct Model(ObjectSubclass<imp::Model>) @implements gio::ListModel;
}

// Constructor for new instances. This simply calls glib::Object::new()
impl Model {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Model {
        glib::Object::new(&[]).expect("Failed to create Model")
    }

    pub fn append(&self, obj: &DownloadStatistics) {
        let imp = self.imp();
        let index = {
            // Borrow the data only once and ensure the borrow guard is dropped
            // before we emit the items_changed signal because the view
            // could call get_item / get_n_item from the signal handler to update its state
            let mut data = imp.0.borrow_mut();
            data.push(obj.clone());
            data.len() - 1
        };
        // Emits a signal that 1 item was added, 0 removed at the position index
        self.items_changed(index as u32, 0, 1);
    }

    // apply closure to the item which has same torrent name as parameter
    pub fn edit(&self, peer_id: &[u8], f: impl Fn(&mut DownloadStatistics)) {
        let imp = self.imp();
        let mut data = imp.0.borrow_mut();
        for item in data.iter_mut() {
            if item.property::<String>("id") == DownloadStatistics::sha1_of(peer_id) {
                f(item);
            }
        }
    }

    pub fn sort_by_download_rate(&self) -> Vec<DownloadStatistics> {
        let imp = self.imp();
        let sorted = &mut *imp.0.borrow_mut().clone();
        sorted.sort_by(|a, b| {
            let a_rate = a.property::<f32>("downloadrate");
            let b_rate = b.property::<f32>("downloadrate");
            a_rate.partial_cmp(&b_rate).unwrap()
        });
        let mut sorted = sorted.to_vec();
        sorted.reverse();
        sorted
    }
    // implement clear using self.remove() only!
    pub fn clear(&self) {
        let len = self.imp().0.borrow().len();
        for _ in 0..len {
            self.remove_by_index(0);
        }
    }

    pub fn edit_state(&self, peer_id: &[u8], peer_conn_state: PeerConnectionState) {
        let _client_interested = match peer_conn_state.client.interested {
            true => "interested",
            false => "not interested",
        };
        let _client_choked = match peer_conn_state.client.chocked {
            true => "choked",
            false => "not choked",
        };

        let peer_interested = match peer_conn_state.peer.interested {
            true => "interested",
            false => "not interested",
        };
        let peer_choked = match peer_conn_state.peer.chocked {
            true => "choked",
            false => "not choked",
        };

        let peer_state = peer_interested.to_string() + " and " + peer_choked;
        let imp = self.imp();
        let mut data = imp.0.borrow_mut();
        for item in data.iter_mut() {
            if item.property::<String>("id") == DownloadStatistics::sha1_of(peer_id) {
                item.set_property("clientstate", &peer_state);
            }
        }
    }

    pub fn remove_by_index(&self, index: usize) {
        let imp = self.imp();
        let mut data = imp.0.borrow_mut();
        data.remove(index);
        self.items_changed(index as u32, 1, 0);
    }

    pub fn remove(&self, peer_id: &[u8]) {
        let imp = self.imp();
        imp.0
            .borrow_mut()
            .retain(|item| item.property::<String>("id") != DownloadStatistics::sha1_of(peer_id));

        // Emits a signal that 1 item was removed, 0 added at the position index
        self.items_changed(0, 1, 0);
    }
}
