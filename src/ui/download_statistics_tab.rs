use crate::peer::PeerConnectionState;

use super::download_statistics_model::Model;
use super::download_statistics_row::DownloadStatistics;
use super::messages::PeerStatistics;
use super::UIMessage;
use gtk::{self};
use gtk::{
    glib::{self, clone},
    prelude::*,
    ResponseType,
};
use gtk::{PolicyType, ScrolledWindow};
pub struct DownloadStatisticsTab {
    pub container: gtk::Box,
    pub model: Model,
}

#[derive(Debug)]
pub enum DownloadStatisticsTabError {
    Error(&'static str),
    ErrorString(String),
}

impl std::convert::From<gtk::Widget> for DownloadStatisticsTabError {
    fn from(widget: gtk::Widget) -> Self {
        DownloadStatisticsTabError::ErrorString(format!("could not get widget {}", widget))
    }
}

impl DownloadStatisticsTab {
    pub fn new(window: &gtk::ApplicationWindow) -> DownloadStatisticsTab {
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
                    .downcast_ref::<DownloadStatistics>()
                    .expect("Row data is of wrong type");
                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 5);

                let summary_box =  gtk::Box::builder()
                .spacing(2)
                .margin(10)
                .orientation(gtk::Orientation::Vertical)
                .halign(gtk::Align::Start)
                .build();

                Self::add_peer_data(&summary_box, item, "IP:", "ipport");
                Self::add_peer_data(&summary_box, item, "Downloaded Pieces:", "downloadedpieces");
                Self::add_peer_data(&summary_box, item, "Download Rate (MBps):", "downloadrate");
                Self::add_peer_data(&summary_box, item, "Client State:", "clientstate");
                // When the info button is clicked, a new modal dialog is created for seeing
                // the corresponding row
                let details_button = gtk::Button::with_label("Details");
                details_button.set_halign(gtk::Align::End);
                details_button.set_widget_name("details-button");
                Self::dialog(&details_button, &window, item);


                hbox.pack_start(&summary_box, true, true, 0);
                hbox.pack_start(&details_button, false, false, 0);
                // When a row is activated (select + enter) we simply emit the clicked
                // signal on the corresponding edit button to open the edit dialog
                box_.connect_activate(clone!(@weak details_button => move |_| {
                    details_button.emit_clicked();
                }));

                box_.add(&hbox);
                box_.show_all();

                box_.upcast::<gtk::Widget>()
            }),
        );

        let backgorund = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        backgorund.set_widget_name("background");
        backgorund.pack_start(&listbox, true, true, 0);
        scrolled_window.add(&backgorund);
        vbox.pack_start(&scrolled_window, true, true, 0);

        DownloadStatisticsTab {
            container: vbox,
            model,
        }
    }

    fn dialog(
        edit_button: &gtk::Button,
        window: &gtk::ApplicationWindow,
        item: &DownloadStatistics,
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

            Self::add_peer_data(&content_area, &item, "Torrent:", "torrentname");
            Self::add_peer_data(&content_area, &item, "Peer ID:", "id");
            Self::add_peer_data(&content_area, &item, "IP:", "ipport");
            Self::add_peer_data(&content_area, &item, "Client State:", "clientstate");
            Self::add_peer_data(&content_area, &item, "Peer State:", "peerstate");
            Self::add_peer_data(&content_area, &item, "Downloaded Pieces:", "downloadedpieces");
            Self::add_peer_data(&content_area, &item, "Download Rate:", "downloadrate");
            Self::add_peer_data(&content_area, &item, "Upload Rate:", "uploadrate");


            dialog.show_all();
        }));
    }

    fn add_peer_data(content_area: &gtk::Box, item: &DownloadStatistics, label: &str, value: &str) {
        let container = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let label = gtk::Label::new(Some(label));
        label.set_widget_name("label-descriptor");
        container.add(&label);
        let label = gtk::Label::new(None);
        item.bind_property(value, &label, "label")
            .flags(glib::BindingFlags::DEFAULT | glib::BindingFlags::SYNC_CREATE)
            .build();
        container.add(&label);
        content_area.add(&container);
    }

    fn bytesps_to_mbps(&self, bps: f32) -> f32 {
        bps / 125000f32
    }

    fn add_peer(&self, peer_statistics: PeerStatistics) -> Result<(), DownloadStatisticsTabError> {
        self.model.append(&DownloadStatistics::new(
            &peer_statistics.torrentname,
            &peer_statistics.peerid,
            &peer_statistics.ip,
            peer_statistics.port,
            peer_statistics.state.client,
            peer_statistics.state.peer,
        ));
        Ok(())
    }

    fn update_download_rate(
        &self,
        rate: f32,
        peer_id: &[u8],
    ) -> Result<(), DownloadStatisticsTabError> {
        self.model.edit(peer_id, |item| {
            if self.bytesps_to_mbps(rate) > 0f32 {
                item.set_property("downloadrate", &self.bytesps_to_mbps(rate));
            }
        });
        Ok(())
    }

    fn update_downloaded_pieces(&self, peer_id: &[u8]) -> Result<(), DownloadStatisticsTabError> {
        self.model.edit(peer_id, |item| {
            let downloaded_pieces = item.property::<u32>("downloadedpieces") + 1;
            item.set_property("downloadedpieces", &downloaded_pieces);
        });
        // self.sort();
        Ok(())
    }

    fn update_upload_rate(
        &self,
        rate: f32,
        peer_id: &[u8],
    ) -> Result<(), DownloadStatisticsTabError> {
        self.model.edit(peer_id, |item| {
            item.set_property("uploadrate", &self.bytesps_to_mbps(rate));
        });
        Ok(())
    }

    fn update_connection_state(
        &self,
        peer_id: &[u8],
        state: PeerConnectionState,
    ) -> Result<(), DownloadStatisticsTabError> {
        self.model.edit_state(peer_id, state);
        Ok(())
    }

    fn sort(&self) {
        let sorted = self.model.sort_by_download_rate();
        // remove all items by index
        self.model.clear();
        for item in sorted {
            self.model.append(&item);
        }
    }

    fn close_connection(&self, peer_id: &[u8]) -> Result<(), DownloadStatisticsTabError> {
        self.model.edit(peer_id, |item| {
            item.set_property("clientstate", &"Disconnected");
            item.set_property("peerstate", &"Disconnected");
            item.set_property("downloadrate", &0f32);
        });
        self.sort();
        Ok(())
    }

    pub fn update(&mut self, message: &UIMessage) -> Result<(), DownloadStatisticsTabError> {
        match message {
            UIMessage::AddPeerStatistics(peer_statistics) => {
                self.add_peer(peer_statistics.clone())?
            }
            UIMessage::PieceDownloaded(_, peer_id) => {
                self.update_downloaded_pieces(peer_id)?;
            }
            UIMessage::UpdatePeerUploadRate(rate, peer_id) => {
                self.update_upload_rate(*rate, peer_id)?;
            }
            UIMessage::UpdatePeerDownloadRate(rate, peer_id) => {
                self.update_download_rate(*rate, peer_id)?;
            }
            UIMessage::UpdateDownloadedPiece(peer_id) => {
                self.update_downloaded_pieces(peer_id)?;
            }
            UIMessage::ClosedConnection(_, peer_id) => {
                self.close_connection(peer_id)?;
            }
            UIMessage::UpdatePeerConnectionState(peer_id, peer_conn_state) => {
                self.update_connection_state(peer_id, peer_conn_state.clone())?;
            }
            _ => {}
        }
        Ok(())
    }
}
