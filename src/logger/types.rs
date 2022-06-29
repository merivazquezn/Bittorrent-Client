use super::constants::*;
use super::errors::LoggerError;
use super::utils::*;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread::JoinHandle;

pub enum LoggerMessage {
    Log(u32),
    Stop,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct Logger {
    sender: Sender<LoggerMessage>,
}

impl Logger {
    pub fn new(dir_path: &str) -> Result<(Self, JoinHandle<()>), LoggerError> {
        let (tx, rx) = mpsc::channel();
        let file: File = create_log_file_in_dir(LOG_FILE_NAME, dir_path)?;
        let builder = std::thread::Builder::new().name("logger worker".to_string());
        let handle = builder
            .spawn(move || {
                let _ = Self::listen(rx, file);
            })
            .map_err(|_| {
                LoggerError::WorkerCreationError(String::from("Logger failed creating the worker"))
            })?;

        Ok((Self { sender: tx }, handle))
    }

    pub fn log_piece(&self, piece: u32) -> Result<(), LoggerError> {
        self.sender.send(LoggerMessage::Log(piece))?;
        Ok(())
    }

    pub fn stop(&self) {
        let _ = self.sender.send(LoggerMessage::Stop);
    }

    fn listen(receiver: Receiver<LoggerMessage>, mut log_file: File) -> Result<(), RecvError> {
        loop {
            let message: LoggerMessage = receiver.recv()?;
            match message {
                LoggerMessage::Log(piece_number) => Logger::log(&mut log_file, piece_number),
                LoggerMessage::Stop => break,
            }
        }
        Ok(())
    }

    fn log(file: &mut File, piece_number: u32) {
        let _ = file.write_all(format!("Received piece: {}\n", piece_number).as_bytes());
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
    use std::io::Read;
    #[allow(unused_imports)]
    use std::thread;

    #[allow(dead_code)]
    pub fn read_lines_from_file(file_path: &str) -> Vec<String> {
        let mut file = File::open(file_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
        lines
    }

    #[test]
    fn when_logger_creates_file_exists() {
        let path = String::from("./src/logger/test_logs");
        let (logger, logger_handle) = Logger::new(&path).unwrap();
        let file_exists =
            std::path::Path::new(&format!("./src/logger/test_logs/{}", LOG_FILE_NAME)).exists();

        logger.stop();
        logger_handle.join().unwrap();

        assert!(file_exists);
    }

    #[test]
    fn when_logging_single_piece_then_logs_exists_in_file() {
        let path = String::from("./src/logger/test_logs/test_1");
        let (logger, logger_handle) = Logger::new(&path).unwrap();

        logger.log_piece(1).unwrap();

        logger.stop();
        logger_handle.join().unwrap();

        let file_contents = read_lines_from_file("./src/logger/test_logs/test_1/download_log.txt");
        assert_eq!(file_contents.len(), 1);
        assert_eq!(file_contents[0], "Received piece: 1");
    }

    #[test]
    fn when_logging_5_pieces_then_all_logs_exists_in_file() {
        let path = String::from("./src/logger/test_logs/test_2");
        let (logger, logger_handle) = Logger::new(&path).unwrap();

        let _ = logger.log_piece(1).unwrap();
        let _ = logger.log_piece(2).unwrap();
        let _ = logger.log_piece(3).unwrap();
        let _ = logger.log_piece(4).unwrap();
        let _ = logger.log_piece(5).unwrap();

        logger.stop();
        logger_handle.join().unwrap();

        let file_contents = read_lines_from_file("./src/logger/test_logs/test_2/download_log.txt");
        assert_eq!(file_contents.len(), 5);
        assert_eq!(file_contents[0], "Received piece: 1");
        assert_eq!(file_contents[1], "Received piece: 2");
        assert_eq!(file_contents[2], "Received piece: 3");
        assert_eq!(file_contents[3], "Received piece: 4");
        assert_eq!(file_contents[4], "Received piece: 5");
    }
}
