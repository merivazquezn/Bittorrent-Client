use glib::subclass::prelude::*;
use gtk::{glib, prelude::*};
use std::cell::RefCell;

// The actual data structure that stores our values. This is not accessible
// directly from the outside.
#[derive(Default)]
pub struct DownloadStatistics {
    torrentname: RefCell<Option<String>>,
    id: RefCell<Option<String>>,
    ipport: RefCell<Option<String>>,
    clientstate: RefCell<Option<String>>,
    peerstate: RefCell<Option<String>>,
    downloadrate: RefCell<Option<String>>,
    uploadrate: RefCell<Option<String>>,
}

// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for DownloadStatistics {
    const NAME: &'static str = "DownloadStatistics";
    type Type = super::DownloadStatistics;
    type ParentType = glib::Object;
}

// The ObjectImpl trait provides the setters/getters for GObject properties.
// Here we need to provide the values that are internally stored back to the
// caller, or store whatever new value the caller is providing.
//
// This maps between the GObject properties and our internal storage of the
// corresponding values of the properties.
impl ObjectImpl for DownloadStatistics {
    fn properties() -> &'static [glib::ParamSpec] {
        use once_cell::sync::Lazy;
        static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
            vec![
                glib::ParamSpecString::new(
                    "name",
                    "Name",
                    "Name",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "id",
                    "ID",
                    "ID",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "ipport",
                    "IPPort",
                    "IPPort",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "clientstate",
                    "ClientState",
                    "ClientState",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "peerstate",
                    "PeerState",
                    "PeerState",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "downloadrate",
                    "DownloadRate",
                    "DownloadRate",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "uploadrate",
                    "UploadRate",
                    "UploadRate",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
            ]
        });

        PROPERTIES.as_ref()
    }

    fn set_property(
        &self,
        _obj: &Self::Type,
        _id: usize,
        value: &glib::Value,
        pspec: &glib::ParamSpec,
    ) {
        match pspec.name() {
            "torrentname" => {
                let name = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.torrentname.replace(name);
            }
            "id" => {
                let id = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.id.replace(id);
            }
            "ipport" => {
                let ipport = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.ipport.replace(ipport);
            }
            "clientstate" => {
                let clientstate = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.clientstate.replace(clientstate);
            }
            "peerstate" => {
                let peerstate = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.peerstate.replace(peerstate);
            }
            "downloadrate" => {
                let downloadrate = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.downloadrate.replace(downloadrate);
            }
            "uploadrate" => {
                let uploadrate = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.uploadrate.replace(uploadrate);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "name" => self.torrentname.borrow().to_value(),
            "id" => self.id.borrow().to_value(),
            "ipport" => self.ipport.borrow().to_value(),
            "clientstate" => self.clientstate.borrow().to_value(),
            "peerstate" => self.peerstate.borrow().to_value(),
            "downloadrate" => self.downloadrate.borrow().to_value(),
            "uploadrate" => self.uploadrate.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}
