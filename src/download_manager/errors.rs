use std::io;

#[derive(Debug)]
pub enum DownloadManagerError {
    IoError(io::Error),
    EmptyPieceError,
    CreateDirectoryError(String),
    CreateFileError(String),
    MissingPieceError(u32),
}

impl From<io::Error> for DownloadManagerError {
    fn from(error: io::Error) -> Self {
        DownloadManagerError::IoError(error)
    }
}

impl std::fmt::Display for DownloadManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DownloadManagerError::IoError(error) => write!(f, "IoError: {}", error),
            DownloadManagerError::EmptyPieceError => {
                write!(f, "Can't save piece with no data in disk")
            }
            DownloadManagerError::CreateDirectoryError(path) => {
                write!(f, "Can't create directory: {}", path)
            }
            DownloadManagerError::CreateFileError(path) => {
                write!(f, "Can't create file: {}", path)
            }
            DownloadManagerError::MissingPieceError(piece_no) => {
                write!(f, "File for piece {} does not exist", piece_no)
            }
        }
    }
}
