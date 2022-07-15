use bittorrent_rustico::metainfo::{self, Info, Metainfo};
use bittorrent_rustico::peer::IPeerMessageServiceError;
use bittorrent_rustico::peer::{PeerMessage, PeerMessageId};
use bittorrent_rustico::server::Server;
use rand::Rng;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    println!("Hello, world!");
}
