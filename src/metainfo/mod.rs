mod errors;
mod parser;
mod types;

pub use errors::MetainfoParserError;
pub use parser::parse;
pub use types::Info;
pub use types::Metainfo;
