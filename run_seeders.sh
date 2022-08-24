#! /bin/bash

# run seeders in background
RUST_LOG=info UI=yes ./peer.exe config_chiken_seeder.txt ./example_torrents/gif.gif.torrent &
RUST_LOG=info UI=yes ./peer.exe config_logo_seeder.txt ./example_torrents/logo512.png.torrent &
RUST_LOG=info UI=yes ./peer.exe config_eduardo_seeder.txt ./example_torrents/eduardo.mp4.torrent &
