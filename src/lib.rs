pub mod application;
pub mod application_errors;
pub mod bencode;
pub mod client;
pub mod config;
pub mod download_manager;
pub mod http;
pub mod logger;
pub mod metainfo;
pub mod peer;
pub mod peer_connection_manager;
pub mod piece_manager;
pub mod piece_saver;
pub mod server;
pub mod tracker;
pub mod ui;

pub mod boxed_result {
    use std::error;
    pub type BoxedResult<T> = std::result::Result<T, Box<dyn error::Error>>;
}
