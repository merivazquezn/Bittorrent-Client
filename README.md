# 22C1-Polleria-Rustiseria

Bittorrent Client in Rust

## Prerequisites

2. install gtk3 dev library for your system

## Running 

example:
```
RUST_LOG=info UI=true cargo run ./example_torrents/debian.torrent
```

If you want to run application without UI, avoid setting the UI environment variable:
```
RUST_LOG=info cargo run ./example_torrents/debian.torrent```

run integration tests:
```
RUST_LOG=trace cargo test --test "*" -- --nocapture
```

## Presentation and Report

there are slides and report available at this repo, that explain in detail how the whole project works.
