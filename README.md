# 22C1-Polleria-Rustiseria

Bittorrent Client in Rust

## Prerequisites

2. install gtk3 dev library for your system

## Running 

example for info:
```
RUST_LOG=info UI=true cargo run debian.torrent
```

example for debug:
```
RUST_LOG=debug UI=true cargo run debian.torrent
```

If you want to run application without UI, avoid setting the UI environment variable:
```
RUST_LOG=debug cargo run debian.torrent
```