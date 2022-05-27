use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
pub enum LoggerError {
    CreateDirectoryError(String),
    CreateFileError(String),
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
        }
    }
}
