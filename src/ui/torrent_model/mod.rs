//! Defines our custom model

mod imp;

use super::torrent_list_row::TorrentInformation;
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

    pub fn append(&self, obj: &TorrentInformation) {
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
    pub fn edit(&self, torrent: &str, f: impl Fn(&mut TorrentInformation)) {
        let imp = self.imp();
        let mut data = imp.0.borrow_mut();
        for item in data.iter_mut() {
            if item.property::<String>("name") == torrent {
                f(item);
            }
        }
    }

    pub fn remove(&self, index: u32) {
        let imp = self.imp();
        imp.0.borrow_mut().remove(index as usize);
        // Emits a signal that 1 item was removed, 0 added at the position index
        self.items_changed(index, 1, 0);
    }
}
