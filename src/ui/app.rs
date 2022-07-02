use super::Notebook;
use super::UIMessage;
use glib::{Continue, PRIORITY_DEFAULT};
use gtk::gdk_pixbuf::PixbufLoader;
use gtk::prelude::*;
use gtk::{self, gdk, glib};
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
        .application_id("org.gtk-rs.bittorrent")
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
        .title("Bittorrent Rústico")
        .default_height(480)
        .default_width(640)
        .build();

    let loader = PixbufLoader::with_type("ico").unwrap();
    loader.write(include_bytes!("resources/logo.ico")).unwrap();
    loader.close().unwrap();
    window.set_icon(Some(&loader.pixbuf().unwrap()));

    let provider = gtk::CssProvider::new();
    let style = include_bytes!("resources/styles.css");
    provider.load_from_data(style).expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let (tx_messages, rx_messages) = glib::MainContext::channel(PRIORITY_DEFAULT);
    client_sender
        .send(tx_messages)
        .expect("could not send sender to client");

    let notebook = Rc::new(RefCell::new(Notebook::new(&window)));

    let notebook_clone = notebook.clone();
    rx_messages.attach(None, move |msg| {
        if let Err(err) = notebook_clone.borrow_mut().update(msg) {
            error!("error updating UI {:?}", err);
        }
        Continue(true)
    });

    window.add(&notebook.borrow().notebook);
    window.show_all();
}
