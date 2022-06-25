use super::Notebook;
use super::UIMessage;
use glib::{Continue, PRIORITY_DEFAULT};
use gtk::prelude::*;
use gtk::{self, glib};
use gtk::{Application, ApplicationWindow};
use log::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc::Sender;
// struct GeneralTorrentInformation {
//     name: String,
//     info_hash: String,
//     length: u64,
//     piece_count: u32,
//     peer_count: u32,
//     completion_percentage: u32,
//     downloaded_verified_piece_count: u32,
//     active_connections: u32,
// }

// struct DownloadStatistics {
//     peerStatistics: Vec<PeerStatistics>,
// }

pub fn run_ui(client_sender: Sender<glib::Sender<UIMessage>>) {
    let app = Application::builder()
        .application_id("org.gtk-rs.example")
        .build();

    app.connect_activate(move |app| {
        build_ui(app, &client_sender);
    });

    let args: Vec<String> = vec![]; // necessary to not use main program args
    app.run_with_args(&args);
}

fn build_ui(app: &Application, client_sender: &Sender<glib::Sender<UIMessage>>) {
    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("My GTK App")
        .build();

    let (tx_messages, rx_messages) = glib::MainContext::channel(PRIORITY_DEFAULT);
    client_sender.send(tx_messages).unwrap();

    let notebook = Rc::new(RefCell::new(Notebook::new(&window)));

    let notebook_clone = notebook.clone();
    rx_messages.attach(None, move |msg| {
        if let Err(err) = notebook_clone.borrow_mut().update(msg) {
            error!("error updating UI {:?}", err);
        }
        Continue(true)
    });

    let gtk_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    gtk_box.add(&notebook.borrow().notebook);

    window.add(&gtk_box);
    window.show_all();
}
