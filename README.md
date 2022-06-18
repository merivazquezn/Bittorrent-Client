# 22C1-Polleria-Rustiseria

Bittorrent Client in Rust

## Prerequisites

2. install gtk4 library. For ubuntu:
```
sudo apt-get install libgtk-4-dev
```
   


## Running 

example for info:
```
RUST_LOG=info UI=true cargo run ubuntu.torrent
```

example for debug:
```
RUST_LOG=debug UI=true cargo run ubuntu.torrent
```

If you want to run application without UI, avoid setting the UI environment variable:
```
RUST_LOG=debug cargo run ubuntu.torrent
```