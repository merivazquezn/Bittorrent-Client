use std::num;
#[derive(Debug, PartialEq)]
/// Errors that can occur when parsing the config file
pub enum ConfigError {
    /// The port is not in range or a valid number
    InvalidPort(num::ParseIntError),
    /// The path is not valid or exists
    InvalidPath(String),
    /// there is a key missing in the config file
    MissingKey(String),
}

impl From<std::num::ParseIntError> for ConfigError {
    fn from(error: std::num::ParseIntError) -> Self {
        ConfigError::InvalidPort(error)
    }
}

// implement display for every type of error
impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::InvalidPort(e) => write!(f, "Invalid port: {}", e),
            ConfigError::InvalidPath(e) => {
                write!(f, "{} is not an existing directory", e)
            }
            ConfigError::MissingKey(key) => write!(f, "Missing key: {}", key),
        }
    }
}
