use crate::config::ConfigError;
use crate::http::HttpsServiceError;
use crate::logger::LoggerError;
use crate::metainfo::MetainfoParserError;
use crate::peer::PeerConnectionError;
use crate::server::ServerError;
use crate::tracker::TrackerError;
use std::fmt;
use std::fmt::Display;

/// The error type that is returned by the application
/// Each error holds inside the error of the exact type
pub enum ApplicationError {
    ConfigError(ConfigError),
    MetainfoError(MetainfoParserError),
    TrackerError(TrackerError),
    HttpsServiceError(HttpsServiceError),
    LoggerError(LoggerError),
    JoinError(String),
    PeerConnectionError(PeerConnectionError),
    ServerError(ServerError),
}

impl From<ServerError> for ApplicationError {
    fn from(error: ServerError) -> Self {
        ApplicationError::ServerError(error)
    }
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

impl From<HttpsServiceError> for ApplicationError {
    fn from(error: HttpsServiceError) -> Self {
        ApplicationError::HttpsServiceError(error)
    }
}

impl From<LoggerError> for ApplicationError {
    fn from(error: LoggerError) -> Self {
        ApplicationError::LoggerError(error)
    }
}

impl From<PeerConnectionError> for ApplicationError {
    fn from(error: PeerConnectionError) -> Self {
        ApplicationError::PeerConnectionError(error)
    }
}

impl From<Box<dyn std::any::Any + std::marker::Send>> for ApplicationError {
    fn from(error: Box<dyn std::any::Any + std::marker::Send>) -> Self {
        ApplicationError::JoinError(format!("{:?}", error))
    }
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::ConfigError(error) => write!(f, "Config Error - {}", error),
            ApplicationError::MetainfoError(error) => write!(f, "Metainfo Error - {}", error),
            ApplicationError::TrackerError(error) => write!(f, "Tracker Error - {}", error),
            ApplicationError::LoggerError(error) => write!(f, "Logger Error - {}", error),
            ApplicationError::HttpsServiceError(error) => {
                return write!(f, "HttpsService Error - {}", error);
            }
            ApplicationError::PeerConnectionError(error) => {
                write!(f, "Peer Connection Error - {}", error)
            }
            ApplicationError::JoinError(cause) => write!(f, "Join Error - {}", cause),
            ApplicationError::ServerError(error) => write!(f, "Server Error - {}", error),
        }
    }
}
