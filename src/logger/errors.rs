use std::fmt::{Display, Formatter, Result};
use std::sync::mpsc::{RecvError, SendError};

#[derive(Debug)]
/// The error type returned when there is a problem with the Logger
pub enum LoggerError {
    /// When logger fails crating a new directory for saving the log file
    CreateDirectoryError(String),

    /// When logger fails creating a file for the log file
    CreateFileError(String),

    /// Throws when logger fails to read the message sent by the sender side of the logger
    ReadingError(RecvError),

    /// Throws when the logger fails to send a message to the LoggerWorker
    InexistentListener,
}

impl<T> From<SendError<T>> for LoggerError {
    fn from(_: SendError<T>) -> Self {
        LoggerError::InexistentListener
    }
}

impl From<RecvError> for LoggerError {
    fn from(error: RecvError) -> Self {
        LoggerError::ReadingError(error)
    }
}

impl Display for LoggerError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            LoggerError::CreateDirectoryError(path) => {
                write!(f, "Can't create directory: {}", path)
            }
            LoggerError::CreateFileError(path) => {
                write!(f, "Can't create file: {}", path)
            }
            LoggerError::InexistentListener => write!(
                f,
                "Logger failed writing because the listener doesn't exist"
            ),
            LoggerError::ReadingError(error) => {
                write!(f, "Logger failed when reading from channel: {}", error)
            }
        }
    }
}
