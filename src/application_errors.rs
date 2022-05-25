use std::fmt;
use std::fmt::Display;
// use all the modules config, peer, tracker, metainfo
use crate::config::ConfigError;
use crate::http::HttpsConnectionError;
use crate::metainfo::MetainfoParserError;
use crate::tracker::TrackerError;

/// The error type that is returned by the application
pub enum ApplicationError {
    ConfigError(ConfigError),
    MetainfoError(MetainfoParserError),
    TrackerError(TrackerError),
    HttpsConnectionError(HttpsConnectionError),
}

impl From<ConfigError> for ApplicationError {
    fn from(error: ConfigError) -> Self {
        ApplicationError::ConfigError(error)
    }
}

impl From<MetainfoParserError> for ApplicationError {
    fn from(error: MetainfoParserError) -> Self {
        ApplicationError::MetainfoError(error)
    }
}

impl From<TrackerError> for ApplicationError {
    fn from(error: TrackerError) -> Self {
        ApplicationError::TrackerError(error)
    }
}

impl From<HttpsConnectionError> for ApplicationError {
    fn from(error: HttpsConnectionError) -> Self {
        ApplicationError::HttpsConnectionError(error)
    }
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::ConfigError(error) => write!(f, "Config Error - {}", error),
            ApplicationError::MetainfoError(error) => write!(f, "Metainfo Error - {}", error),
            ApplicationError::TrackerError(error) => write!(f, "Tracker Error - {}", error),
            ApplicationError::HttpsConnectionError(error) => {
                return write!(f, "HttpsConnection Error - {}", error);
            }
        }
    }
}
