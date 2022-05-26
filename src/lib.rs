pub mod application;
pub mod application_errors;
pub mod bencode;
pub mod config;
pub mod download_manager;
pub mod http;
pub mod metainfo;
pub mod peer;
// pub mod tcp;
pub mod tracker;

pub mod boxed_result {
    use std::error;
    pub type BoxedResult<T> = std::result::Result<T, Box<dyn error::Error>>;
}
