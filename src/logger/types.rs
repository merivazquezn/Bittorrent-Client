use super::constants::*;
use super::errors::LoggerError;
use super::utils::*;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc;

#[allow(dead_code)]
pub struct Logger {
    sender: mpsc::Sender<u32>,
}

pub struct LoggerListener {
    receiver: mpsc::Receiver<u32>,
    file: File,
}

impl Logger {
    pub fn new(dir_path: &str) -> Result<(Self, LoggerListener), LoggerError> {
        let file: File = create_log_file_in_dir(LOG_FILE_NAME, dir_path)?;
        let (tx, rx) = mpsc::channel();

        let logger = Logger { sender: tx };

        let logger_listener = LoggerListener { receiver: rx, file };

        Ok((logger, logger_listener))
    }

    pub fn log_piece(&self, piece_number: u32) {
        self.sender.send(piece_number).unwrap();
    }
}

impl LoggerListener {
    pub fn listen(&mut self) {
        loop {
            if let Ok(piece_number) = self.receiver.recv() {
                self.log(piece_number);
            }
        }
    }

    fn log(&mut self, piece_number: u32) {
        let _ = self
            .file
            .write_all(format!("Received piece: {}\n", piece_number).as_bytes());
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn when_logger_creates_file_exists() {
        let path = String::from("./src/logger/test_logs");
        let _logger = Logger::new(&path).unwrap();
        let file_exists =
            std::path::Path::new(&format!("./src/logger/test_logs/{}", LOG_FILE_NAME)).exists();
        assert!(file_exists);
    }
}
