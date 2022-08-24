# 22C1-Polleria-Rustiseria

Bittorrent Client in Rust

## Group Members
- Luciano Trujillo
- Tomás Szwarcberg
- María Vázquez Navarro
- Matías Fusco

## Prerequisites

2. install gtk3 dev library for your system

## Running as executable

1. from /tracker, run:
./tracker.exe

2. from the root of the repo, run:
./peer.exe <config file path> <torrent1> <torrent2> ...

## Running simulation of multiple peers and torrents

1. from /tracker, run:
./tracker.exe

2. from the root of the repo, run:
./run_simulation.sh

## Running with cargo run

example:
```
RUST_LOG=info UI=true cargo run ./example_torrents/debian.torrent
```

If you want to run application without UI, avoid setting the UI environment variable:
```
RUST_LOG=info cargo run ./example_torrents/debian.torrent

run integration tests:
```
RUST_LOG=trace cargo test --test "*" -- --nocapture
```

## Presentation and Report

there are slides and report available at this repo, that explain in detail how the whole project works.
