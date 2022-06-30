use super::torrent_list_row::TorrentInformation;
use super::torrent_model::Model;
use super::UIMessage;
use crate::metainfo::Metainfo;
use gtk::{self};
use gtk::{
    glib::{self, clone},
    prelude::*,
    ResponseType,
};
use gtk::{PolicyType, ScrolledWindow};
use sha1::{Digest, Sha1};

pub struct GeneralInformationTab {
    pub container: gtk::Box,
    pub model: Model,
}

#[derive(Debug)]
pub enum GeneralInformationTabError {
    Error(&'static str),
    ErrorString(String),
}

impl std::convert::From<gtk::Widget> for GeneralInformationTabError {
    fn from(widget: gtk::Widget) -> Self {
        GeneralInformationTabError::ErrorString(format!("could not get widget {}", widget))
    }
}

impl GeneralInformationTab {
    pub fn new(window: &gtk::ApplicationWindow) -> GeneralInformationTab {
        let model = Model::new();
        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);

        let scrolled_window = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never) // Disable horizontal scrolling
            .min_content_height(1400)
            .build();

        let listbox = gtk::ListBox::new();
        listbox.bind_model(
            Some(&model),
            clone!(@weak window => @default-panic,  move |item| {
                let box_ = gtk::ListBoxRow::new();
                let item = item
                    .downcast_ref::<TorrentInformation>()
                    .expect("Row data is of wrong type");
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

                let label = gtk::Label::new(None);
                item.bind_property("name", &label, "label")
                    .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                    .build();
                hbox.pack_start(&label, false, false, 0);
                let label = gtk::Label::new(None);
                item.bind_property("infohash", &label, "label")
                    .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
                    .build();
                hbox.pack_start(&label, false, false, 0);

                // When the info button is clicked, a new modal dialog is created for seeing
                // the corresponding row
                let edit_button = gtk::Button::with_label("Info");
                Self::dialog(&edit_button, &window, item);

                hbox.pack_start(&edit_button, false, false, 0);
                box_.add(&hbox);

                // When a row is activated (select + enter) we simply emit the clicked
                // signal on the corresponding edit button to open the edit dialog
                box_.connect_activate(clone!(@weak edit_button => move |_| {
                    edit_button.emit_clicked();
                }));

                box_.show_all();

                box_.upcast::<gtk::Widget>()
            }),
        );

        scrolled_window.add(&listbox);
        vbox.pack_start(&scrolled_window, true, true, 0);

        GeneralInformationTab {
            container: vbox,
            model,
        }
    }

    fn dialog(
        edit_button: &gtk::Button,
        window: &gtk::ApplicationWindow,
        item: &TorrentInformation,
    ) {
        edit_button.connect_clicked(clone!(@weak window, @strong item => move |_| {
            let dialog = gtk::Dialog::builder()
                .title("Edit Item")
                .parent(&window)
                .default_height(400)
                .default_width(400)
                .build();

            dialog.add_button("Close", ResponseType::Close);
            dialog.set_default_response(ResponseType::Close);
            dialog.connect_response(|dialog, _| dialog.close());

            let content_area = dialog.content_area();

            Self::add_torrent_data(&content_area, &item, "Name: ", "name");
            Self::add_torrent_data(&content_area, &item, "Verification Hash: ", "infohash");
            Self::add_torrent_data(&content_area, &item, "Total Size in MB: ", "totalsize");
            Self::add_torrent_data(&content_area, &item, "Total Piece Count: ", "totalpiececount");
            Self::add_torrent_data(&content_area, &item, "Peer Count: ", "peercount");
            Self::add_torrent_data(&content_area, &item, "Downloaded Pieces: ", "downloadedpieces");
            Self::add_torrent_data(&content_area, &item, "Active Connections: ", "activeconnections");
            Self::add_torrent_data(&content_area, &item, "File Structure: ", "filestructure");
            Self::add_torrent_percentage(&content_area, &item, "Download progress: ", "downloadpercentage");

            dialog.show_all();
        }));
    }

    fn add_torrent_data(
        content_area: &gtk::Box,
        item: &TorrentInformation,
        label: &str,
        value: &str,
    ) {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        container.add(&gtk::Label::new(Some(label)));
        let label = gtk::Label::new(None);
        item.bind_property(value, &label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        container.add(&label);
        content_area.add(&container);
    }

    fn add_torrent_percentage(
        content_area: &gtk::Box,
        item: &TorrentInformation,
        label: &str,
        value: &str,
    ) {
        let progress_bar = gtk::ProgressBar::builder()
            .show_text(true)
            .text(label)
            .hexpand(true)
            .build();
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        item.bind_property(value, &progress_bar, "fraction")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        container.add(&progress_bar);
        content_area.add(&container);
    }

    // function that converts Vec<u8> bytes to ascii characters
    fn bytes_to_ascii(&self, bytes: &[u8]) -> String {
        format!("{:02x?}", bytes)
            .replace('[', "")
            .replace(']', "")
            .replace(", ", "")
    }

    fn sha1_of(&self, vec: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(vec);
        self.bytes_to_ascii(&hasher.finalize())
    }

    fn bytes_to_megabytes(&self, bytes: u64) -> u64 {
        bytes / 1024 / 1024
    }

    fn add_torrent(&self, metainfo: &Metainfo) -> Result<(), GeneralInformationTabError> {
        self.model.append(&TorrentInformation::new(
            &metainfo.info.name,
            &self.sha1_of(&metainfo.info_hash),
            self.bytes_to_megabytes(metainfo.info.length),
            metainfo.info.pieces.len() as u32,
            &metainfo.info.name,
        ));
        Ok(())
    }

    fn set_initial_torrent_peers(
        &self,
        torrent: &str,
        amount: u32,
    ) -> Result<(), GeneralInformationTabError> {
        self.model.edit(torrent, |item| {
            item.set_property("peercount", &amount);
        });
        Ok(())
    }

    fn add_connection_to_torrent(&self, torrent: &str) -> Result<(), GeneralInformationTabError> {
        self.model.edit(torrent, |item| {
            let active_connections = item.property::<u32>("activeconnections") + 1;
            item.set_property("activeconnections", &active_connections);
        });
        Ok(())
    }

    fn piece_downloaded(&self, torrent: &str) -> Result<(), GeneralInformationTabError> {
        self.model.edit(torrent, |item| {
            let downloaded_pieces = item.property::<u32>("downloadedpieces") + 1;
            let download_percentage: f32 =
                (downloaded_pieces) as f32 / item.property::<u32>("totalpiececount") as f32;
            item.set_property("downloadedpieces", &downloaded_pieces);
            item.set_property("downloadpercentage", &download_percentage);
        });
        Ok(())
    }
    fn closed_connection_to_torrent(
        &self,
        torrent: &str,
    ) -> Result<(), GeneralInformationTabError> {
        self.model.edit(torrent, |item| {
            let active_connections = item.property::<u32>("activeconnections") - 1;
            item.set_property("activeconnections", &active_connections);
        });
        Ok(())
    }

    pub fn update(&mut self, message: &UIMessage) -> Result<(), GeneralInformationTabError> {
        match message {
            UIMessage::AddTorrent(metainfo) => self.add_torrent(metainfo)?,
            UIMessage::NewConnection(torrent) => self.add_connection_to_torrent(torrent)?,
            UIMessage::ClosedConnection(torrent) => self.closed_connection_to_torrent(torrent)?,
            UIMessage::PieceDownloaded(torrent) => self.piece_downloaded(torrent)?,
            UIMessage::TorrentInitialPeers(torrent, amount) => {
                self.set_initial_torrent_peers(torrent, *amount)?
            }
            _ => unreachable!(),
        }
        Ok(())
    }
}
