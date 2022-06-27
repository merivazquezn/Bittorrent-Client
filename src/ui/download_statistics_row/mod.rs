//! Our GObject subclass for carrying a name and count for the ListBox model
//!
//! Both name and count are stored in a RefCell to allow for interior mutability
//! and are exposed via normal GObject properties. This allows us to use property
//! bindings below to bind the values with what widgets display in the UI

mod imp;

use gtk::glib;

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
        id: &str,
        ipport: &str,
        clientstate: &str,
        peerstate: &str,
    ) -> DownloadStatistics {
        glib::Object::new(&[
            ("torrentname", &torrentname),
            ("id", &id),
            ("ipport", &ipport),
            ("clientstate", &clientstate),
            ("peerstate", &peerstate),
        ])
        .expect("Failed to create row data")
    }
}
