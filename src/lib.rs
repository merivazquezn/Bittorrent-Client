pub mod bencode;
pub mod config;
pub mod metainfo;
pub mod torrent_parser;

pub fn run() -> Result<(), config::ConfigError> {
    Ok(())
}
