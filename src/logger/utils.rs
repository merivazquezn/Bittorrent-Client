use super::errors::LoggerError;
use std::fs;
use std::path::Path;

fn create_downloads_directory(path: &str) -> Result<(), LoggerError> {
    if !path.is_empty() && !Path::new(path).exists() {
        fs::create_dir_all(path)
            .map_err(|_| LoggerError::CreateDirectoryError(path.to_string()))?;
    }

    Ok(())
}

// Creates a log file in the given directory, creating the directory if it does not exist.
pub fn create_log_file_in_dir(
    file_name: &str,
    dir_path: &str,
) -> Result<std::fs::File, LoggerError> {
    create_downloads_directory(dir_path)?;

    let log_file = fs::File::create(format!("{}/{}", dir_path, file_name))
        .map_err(|_| LoggerError::CreateFileError(format!("{}/{}", dir_path, file_name)))?;

    Ok(log_file)
}
