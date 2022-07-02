use glib::subclass::prelude::*;
use gtk::{glib, prelude::*};
use std::cell::RefCell;

// The actual data structure that stores our values. This is not accessible
// directly from the outside.
#[derive(Default)]
pub struct TorrentInformation {
    name: RefCell<Option<String>>,
    infohash: RefCell<Option<String>>,
    totalsize: RefCell<u64>,
    totalpiececount: RefCell<u32>,
    peercount: RefCell<u32>,
    downloadpercentage: RefCell<f32>,
    downloadedpieces: RefCell<u32>,
    activeconnections: RefCell<u32>,
    filestructure: RefCell<Option<String>>,
    timeleft: RefCell<Option<String>>,
}

// Basic declaration of our type for the GObject type system
#[glib::object_subclass]
impl ObjectSubclass for TorrentInformation {
    const NAME: &'static str = "TorrentInformation";
    type Type = super::TorrentInformation;
    type ParentType = glib::Object;
}

// The ObjectImpl trait provides the setters/getters for GObject properties.
// Here we need to provide the values that are internally stored back to the
// caller, or store whatever new value the caller is providing.
//
// This maps between the GObject properties and our internal storage of the
// corresponding values of the properties.
impl ObjectImpl for TorrentInformation {
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
                    "infohash",
                    "InfoHash",
                    "InfoHash",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecUInt64::new(
                    "totalsize",
                    "TotalSize",
                    "TotalSize",
                    0,
                    u64::MAX,
                    0, // Allowed range and default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecUInt::new(
                    "totalpiececount",
                    "TotalPieceCount",
                    "TotalPieceCount",
                    0,
                    u32::MAX,
                    0, // Allowed range and default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecUInt::new(
                    "peercount",
                    "PeerCount",
                    "PeerCount",
                    0,
                    u32::MAX,
                    0, // Allowed range and default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecFloat::new(
                    "downloadpercentage",
                    "DownloadPercentage",
                    "DownloadPercentage",
                    0.0,
                    100.0,
                    0.0, // Allowed range and default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecUInt::new(
                    "downloadedpieces",
                    "DownloadedPieces",
                    "DownloadedPieces",
                    0,
                    u32::MAX,
                    0, // Allowed range and default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecUInt::new(
                    "activeconnections",
                    "ActiveConnections",
                    "ActiveConnections",
                    0,
                    u32::MAX,
                    0, // Allowed range and default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "timeleft",
                    "TimeLeft",
                    "TimeLeft",
                    None, // Default value
                    glib::ParamFlags::READWRITE,
                ),
                glib::ParamSpecString::new(
                    "filestructure",
                    "FileStructure",
                    "FileStructure",
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
            "name" => {
                let name = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.name.replace(name);
            }
            "infohash" => {
                let infohash = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.infohash.replace(infohash);
            }
            "totalsize" => {
                let totalsize = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.totalsize.replace(totalsize);
            }
            "totalpiececount" => {
                let totalpiececount = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.totalpiececount.replace(totalpiececount);
            }
            "peercount" => {
                let peercount = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.peercount.replace(peercount);
            }
            "downloadpercentage" => {
                let downloadpercentage = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.downloadpercentage.replace(downloadpercentage);
            }
            "downloadedpieces" => {
                let downloadedpieces = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.downloadedpieces.replace(downloadedpieces);
            }
            "activeconnections" => {
                let activeconnections = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.activeconnections.replace(activeconnections);
            }
            "timeleft" => {
                let timeleft = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.timeleft.replace(timeleft);
            }
            "filestructure" => {
                let filestructure = value
                    .get()
                    .expect("type conformity checked by `Object::set_property`");
                self.filestructure.replace(filestructure);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
        match pspec.name() {
            "name" => self.name.borrow().to_value(),
            "infohash" => self.infohash.borrow().to_value(),
            "totalsize" => self.totalsize.borrow().to_value(),
            "totalpiececount" => self.totalpiececount.borrow().to_value(),
            "peercount" => self.peercount.borrow().to_value(),
            "downloadpercentage" => self.downloadpercentage.borrow().to_value(),
            "downloadedpieces" => self.downloadedpieces.borrow().to_value(),
            "activeconnections" => self.activeconnections.borrow().to_value(),
            "timeleft" => self.timeleft.borrow().to_value(),
            "filestructure" => self.filestructure.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}
