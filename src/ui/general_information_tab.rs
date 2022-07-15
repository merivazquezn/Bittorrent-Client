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

use crate::metainfo::File;

pub struct GeneralInformationTab {
    pub container: gtk::Box,
    pub model: Model,
    pub start_time: std::time::Instant,
}
pub struct Directory {
    name: String,
    files: Vec<String>,
    directories: Vec<Directory>,
}

impl Directory {
    pub fn directory_to_string_rec(&self, result: &mut String) {
        result.push_str(&self.name);
        result.push('/');
        for file in &self.files {
            result.push_str(file);
            result.push('\n');
        }
        for directory in &self.directories {
            directory.directory_to_string_rec(result);
        }
    }

    pub fn directory_to_string(&self) -> String {
        let mut string = String::from("");
        self.directory_to_string_rec(&mut string);
        string
    }
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
            .overlay_scrolling(true)
            .vexpand(true)
            .build();
        let listbox = gtk::ListBox::new();
        listbox.bind_model(
            Some(&model),
            clone!(@weak window => @default-panic,  move |item| {
                let box_ = gtk::ListBoxRow::new();
                box_.set_widget_name("listboxrow");
                let item = item
                    .downcast_ref::<TorrentInformation>()
                    .expect("Row data is of wrong type");
                let hbox =  gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .build();
                let summary_box =  gtk::Box::builder()
                .spacing(2)
                .margin(10)
                .orientation(gtk::Orientation::Vertical)
                .halign(gtk::Align::Start)
                .build();
                Self::add_torrent_data(&summary_box, item, "torrent:", "name");
                Self::add_torrent_data(&summary_box, item, "active peers:", "activeconnections");
                Self::add_torrent_data(&summary_box, item, "time left:", "timeleft");
                Self::add_torrent_percentage(&summary_box, item, "Download progress: ", "downloadfraction");

                // When the info button is clicked, a new modal dialog is created for seeing
                // the corresponding row
                let details_button = gtk::Button::with_label("Details");
                details_button.set_halign(gtk::Align::End);
                details_button.set_widget_name("details-button");
                Self::dialog(&details_button, &window, item);

                hbox.pack_start(&summary_box, true, true, 0);
                hbox.pack_start(&details_button, false, false, 0);
                box_.add(&hbox);

                // When a row is activated (select + enter) we simply emit the clicked
                // signal on the corresponding edit button to open the edit dialog
                box_.connect_activate(clone!(@weak details_button => move |_| {
                    details_button.emit_clicked();
                }));

                box_.show_all();

                box_.upcast::<gtk::Widget>()
            }),
        );

        let backgorund = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        backgorund.set_widget_name("background");
        backgorund.pack_start(&listbox, true, true, 0);
        scrolled_window.add(&backgorund);
        vbox.pack_start(&scrolled_window, true, true, 0);

        GeneralInformationTab {
            container: vbox,
            model,
            start_time: std::time::Instant::now(),
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
                .build();

            dialog.add_button("Close", ResponseType::Close);
            dialog.set_default_response(ResponseType::Close);
            dialog.connect_response(|dialog, _| dialog.close());

            let content_area = dialog.content_area();
            content_area.set_widget_name("dialog");
            Self::add_torrent_data(&content_area, &item, "Name: ", "name");
            Self::add_torrent_data(&content_area, &item, "Verification Hash: ", "infohash");
            Self::add_torrent_data(&content_area, &item, "Total Size in MB: ", "totalsize");
            Self::add_torrent_data(&content_area, &item, "Total Piece Count: ", "totalpiececount");
            Self::add_torrent_data(&content_area, &item, "Peer Count: ", "peercount");
            Self::add_torrent_data(&content_area, &item, "Downloaded Pieces: ", "downloadedpieces");
            Self::add_torrent_data(&content_area, &item, "Active Connections: ", "activeconnections");
            Self::add_torrent_data(&content_area, &item, "File Structure: ", "filestructure");
            Self::add_torrent_percentage(&content_area, &item, "Download progress: ", "downloadfraction");
            Self::add_torrent_data(&content_area, &item, "Time taken: ", "timetaken");


            dialog.show_all();
        }));
    }

    fn add_torrent_data(
        content_area: &gtk::Box,
        item: &TorrentInformation,
        label: &str,
        value: &str,
    ) {
        let container = gtk::Box::builder()
            .spacing(10)
            .halign(gtk::Align::Start)
            .build();

        let label = gtk::Label::new(Some(label));
        label.set_widget_name("label-descriptor");

        container.add(&label);
        let label = gtk::Label::builder().halign(gtk::Align::Start).build();
        item.bind_property(value, &label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        container.pack_start(&label, false, false, 0);
        content_area.pack_start(&container, false, false, 0);
    }

    fn add_torrent_percentage(
        content_area: &gtk::Box,
        item: &TorrentInformation,
        label: &str,
        value: &str,
    ) {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 5);

        let description_label = gtk::Label::new(Some(label));
        description_label.set_widget_name("label-descriptor");

        let progress_bar = gtk::ProgressBar::builder()
            .hexpand(true)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .build();
        item.bind_property(value, &progress_bar, "fraction")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        let percentage_label = gtk::Label::new(Some("%"));

        let percentage_value_label = gtk::Label::builder().halign(gtk::Align::End).build();
        item.bind_property("downloadpercentage", &percentage_value_label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();

        container.add(&description_label);
        container.add(&progress_bar);
        container.add(&percentage_value_label);
        container.add(&percentage_label);
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

    // directory struct is a data structure for representing a file structure,
    // it can have nested directories and multiple files per directory

    // converts a list of file paths to a string
    // if the file is a directory, it will be represented as such
    // if the file is a file, it will be represented as such
    // this means that files should be idented "inside" their parent directory
    // the result is a string that can be used to display the file structure

    // each "File" is a struct that has a "path" attribute, which holds the path and name of the file
    // for example: "C:\Users\User\Downloads\file.txt", the director would be "C:\Users\User\Downloads"
    // and the file would be "file.txt"
    fn files_to_string(&self, files: Option<Vec<File>>, name: String) -> String {
        if let Some(files) = files {
            let directory = self.create_directory_structure(files);
            directory.directory_to_string()
        } else {
            name
        }
    }

    // function that recursively creates a directory structure out of Vec<File>
    // util function for files_to_string
    fn create_directory_structure(&self, files: Vec<File>) -> Directory {
        let mut directory = Directory {
            name: String::new(),
            files: Vec::new(),
            directories: Vec::new(),
        };

        for file in &files {
            if file.is_directory() {
                directory.directories.push(self.create_directory_structure(
                    self.get_all_files_or_directories_inside_directory(&file.path, &files),
                ));
            } else {
                directory.files.push(file.name());
            }
        }

        directory
    }

    fn get_all_files_or_directories_inside_directory(
        &self,
        path: &str,
        files: &[File],
    ) -> Vec<File> {
        let mut files_inside_directory = Vec::new();

        for file in files {
            if file.path.clone().contains(path) {
                files_inside_directory.push(file.clone());
            }
        }

        files_inside_directory
    }

    fn add_torrent(&self, metainfo: &Metainfo) -> Result<(), GeneralInformationTabError> {
        self.model.append(&TorrentInformation::new(
            &metainfo.info.name,
            &self.sha1_of(&metainfo.info_hash),
            self.bytes_to_megabytes(metainfo.info.length),
            metainfo.info.pieces.len() as u32,
            &self.files_to_string(metainfo.info.files.clone(), metainfo.info.name.clone()),
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

    fn seconds_to_hh_mm_ss(&self, seconds: u32) -> String {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let seconds = seconds % 60;
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    fn piece_downloaded(&self, torrent: &str) -> Result<(), GeneralInformationTabError> {
        self.model.edit(torrent, |item| {
            let downloaded_pieces = item.property::<u32>("downloadedpieces") + 1;
            let download_fraction: f32 =
                (downloaded_pieces) as f32 / item.property::<u32>("totalpiececount") as f32;
            item.set_property("downloadedpieces", &downloaded_pieces);
            item.set_property("downloadfraction", &download_fraction);
            item.set_property("downloadpercentage", &download_fraction * 100.0);
            let download_speed = item.property::<u32>("downloadedpieces") as f32
                / self.start_time.elapsed().as_secs() as f32;
            let pieces_left = item.property::<u32>("totalpiececount") - downloaded_pieces;
            if download_speed > 0f32 {
                let seconds_left = pieces_left as f32 / download_speed;
                item.set_property("timeleft", self.seconds_to_hh_mm_ss(seconds_left as u32));
            }

            // set time taken to download
            let time_taken = self.start_time.elapsed().as_secs();
            item.set_property("timetaken", self.seconds_to_hh_mm_ss(time_taken as u32));
        });
        Ok(())
    }
    fn closed_connection_to_torrent(
        &self,
        torrent: &str,
    ) -> Result<(), GeneralInformationTabError> {
        self.model.edit(torrent, |item| {
            let mut active_connections = 0;
            if item.property::<u32>("activeconnections") > 0 {
                active_connections = item.property::<u32>("activeconnections") - 1;
            }
            item.set_property("activeconnections", &active_connections);
        });
        Ok(())
    }

    pub fn update(&mut self, message: &UIMessage) -> Result<(), GeneralInformationTabError> {
        match message {
            UIMessage::AddTorrent(metainfo) => self.add_torrent(metainfo)?,
            UIMessage::NewConnection(torrent) => self.add_connection_to_torrent(torrent)?,
            UIMessage::ClosedConnection(torrent, _) => {
                self.closed_connection_to_torrent(torrent)?
            }
            UIMessage::PieceDownloaded(torrent, _) => {
                self.piece_downloaded(torrent)?;
            }
            UIMessage::TorrentInitialPeers(torrent, amount) => {
                self.set_initial_torrent_peers(torrent, *amount)?
            }
            _ => {}
        }
        Ok(())
    }
}
