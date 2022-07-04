use bittorrent_rustico::application::run_with_torrent;
use bittorrent_rustico::ui::{run_ui, UIMessage};
use gtk::{self, glib};
use log::*;
use std::env;
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
fn main() {
    pretty_env_logger::init();
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
    let args = env::args().skip(1);
    // iterate through all args and call run_with_torrent for each torrent file
    let mut torrent_handles: Vec<JoinHandle<()>> = vec![];
    for torrent_file in args {
        info!("Running with torrent file: {}", torrent_file);
        let ui_msg_sender_clone = ui_message_sender.clone();
        let torrent_file = torrent_file.to_string();
        torrent_handles.push(thread::spawn(move || {
            if let Err(err) = run_with_torrent(&torrent_file, ui_msg_sender_clone) {
                error!("Error running with torrent file: {}", torrent_file);
                error!("{}", err);
            }
        }));
    }

    for torrent_handle in torrent_handles {
        torrent_handle.join().unwrap();
    }

    info!("Finished running");
}
