use bittorrent_rustico::application::run_with_torrent;
use log::*;
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    match args.next() {
        Some(torrent_path) => {
            if let Err(e) = run_with_torrent(&torrent_path) {
                error!("{}", e);
            }
        }
        None => error!("Please provide torrent path"),
    }
}
