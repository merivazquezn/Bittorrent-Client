use bittorrent_rustico::application::run_with_torrent;
use bittorrent_rustico::ui::{run_ui, UIMessage};
use gtk::{self, glib};
use log::*;
use std::env;
use std::sync::mpsc;
use std::thread;
fn main() {
    if env::var("UI").is_ok() {
        run_client_with_ui();
    } else {
        run_client_with_no_ui();
    }
}

fn run_client_with_no_ui() {
    run_client(None);
}

fn run_client_with_ui() {
    let (client_sender, client_receiver) = mpsc::channel(); // channel necessary to pass the ui sender to the client
    let client_handle = thread::spawn(move || {
        let ui_tx = client_receiver.recv().unwrap(); // receive the ui sender from the client
        run_client(Some(ui_tx)); // run the client with the ui sender
    });
    run_ui(client_sender);
    client_handle.join().unwrap();
}

fn run_client(ui_message_sender: Option<glib::Sender<UIMessage>>) {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(torrent_path) => {
            if let Err(e) = run_with_torrent(&torrent_path, ui_message_sender) {
                error!("{}", e);
            }
        }
        None => error!("Please provide torrent path"),
    }
}
