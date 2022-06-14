use bittorrent_rustico::application::run_with_torrent;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button};
use log::*;
use std::env;
use std::thread;

fn main() {
    let ui_handle = thread::spawn(move || {
        let app = Application::builder()
            .application_id("org.gtk-rs.example")
            .build();

        app.connect_activate(build_ui);

        let args: Vec<String> = vec![]; // necessary to not use main program args
        app.run_with_args(&args);
    });

    let mut args = env::args().skip(1);
    match args.next() {
        Some(torrent_path) => {
            if let Err(e) = run_with_torrent(&torrent_path) {
                error!("{}", e);
            }
        }
        None => error!("Please provide torrent path"),
    }

    ui_handle.join().unwrap();
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("First GTK Program")
        .default_width(350)
        .default_height(70)
        .build();

    let button = Button::with_label("Click me!");
    button.connect_clicked(|_| {
        eprintln!("Clicked!");
    });
    window.add(&button);

    window.show_all();

    window.present();
}
