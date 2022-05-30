use super::constants::*;
use super::errors::LoggerError;
use super::utils::*;
use log::*;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc;

#[allow(dead_code)]
/// Struct representing the writing side of the logger
pub struct Logger {
    sender: mpsc::Sender<u32>,
    logging_state_sender: mpsc::Sender<LoggerState>,
}

/// Struct representing the listening side of the logger
pub struct LoggerWorker {
    receiver: mpsc::Receiver<u32>,
    logging_state_receiver: mpsc::Receiver<LoggerState>,
    file: File,
}

// Used to send the Logger worker a stop message when neccesary
#[derive(Debug)]
enum LoggerState {
    Continue,
    Stop,
}

/// Implementation of the Writing-side of the logger
impl Logger {
    /// Creates a new logger
    /// Receives the path to the log file, the log file will be named
    ///
    /// # On Succes
    /// Returns a tuple with each side of the logger
    /// The [`LoggerWorker`] should be saved as mutable
    ///  
    /// # On Error
    /// Returns a [`LoggerError`] which holds the reason of failure
    ///
    /// # Example
    /// ```
    /// use bittorrent_rustico::logger::{Logger, LoggerWorker};
    ///
    /// let (logger, mut logger_worker) = Logger::new("./src/logger/test/logs/doc_test").unwrap();
    ///
    /// ```
    pub fn new(dir_path: &str) -> Result<(Self, LoggerWorker), LoggerError> {
        let file: File = create_log_file_in_dir(LOG_FILE_NAME, dir_path)?;
        let (tx, rx) = mpsc::channel();
        let (tx_state, rx_state) = mpsc::channel();

        let logger = Logger {
            sender: tx,
            logging_state_sender: tx_state,
        };

        let logger_listener = LoggerWorker {
            receiver: rx,
            logging_state_receiver: rx_state,
            file,
        };

        Ok((logger, logger_listener))
    }

    pub fn log_piece(&self, piece_number: u32) -> Result<(), LoggerError> {
        self.logging_state_sender.send(LoggerState::Continue)?;
        let _ = self.sender.send(piece_number)?;
        Ok(())
    }

    /// Stops the logger worker
    /// Smoothly closes de file in which the logger worker was writing
    ///
    /// # On Error
    /// Returns a [`LoggerError`] which holds the reason of failure
    ///
    /// # Example
    /// ```
    /// use bittorrent_rustico::logger::{Logger, LoggerWorker};
    ///
    /// let (logger, mut logger_worker) = Logger::new("./src/logger/test/logs/doc_test").unwrap();
    /// let join_handle = std::thread::spawn(move || {
    ///    logger_worker.listen();
    /// });
    ///
    /// logger.stop_logging().unwrap();
    /// join_handle.join().unwrap();
    /// ```
    ///
    pub fn stop_logging(&self) -> Result<(), LoggerError> {
        let _ = self.logging_state_sender.send(LoggerState::Stop)?;
        Ok(())
    }
}

impl LoggerWorker {
    pub fn listen(&mut self) -> Result<(), LoggerError> {
        loop {
            trace!("LoggerWorker: Waiting for message");
            let state = self.logging_state_receiver.recv()?;
            trace!("LoggerWorker: Received message {:?}", state);
            match state {
                LoggerState::Continue => {
                    let piece_number = self.receiver.recv()?;
                    self.log_piece(piece_number);
                }
                LoggerState::Stop => {
                    break;
                }
            }
        }
        Ok(())
    }

    /// Tells the Logger Worker to log that a piece was received, indicating its piece number
    /// The Worker then logs it into the log file
    ///
    /// ```
    /// use bittorrent_rustico::logger::{Logger, LoggerWorker};
    ///
    /// let (logger, mut logger_worker) = Logger::new("./src/logger/test/logs/doc_test").unwrap();
    /// let handle_join = std::thread::spawn(move || {
    ///    logger_worker.listen();
    /// });
    ///
    /// // Receives piece number 1
    /// logger.log_piece(1).unwrap();
    ///
    /// logger.stop_logging().unwrap();
    /// handle_join.join().unwrap();
    ///
    /// ```
    fn log_piece(&mut self, piece_number: u32) {
        let _ = self
            .file
            .write_all(format!("Received piece: {}\n", piece_number).as_bytes());
    }
}

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
        let _ = Logger::new(&path).unwrap();
        let file_exists =
            std::path::Path::new(&format!("./src/logger/test_logs/{}", LOG_FILE_NAME)).exists();
        assert!(file_exists);
    }

    #[test]
    fn when_logging_single_piece_then_logs_exists_in_file() {
        let path = String::from("./src/logger/test_logs/test_1");
        let (logger, mut worker) = Logger::new(&path).unwrap();

        let handle = std::thread::spawn(move || {
            let _ = worker.listen();
        });

        let _ = logger.log_piece(1).unwrap();

        logger.stop_logging().unwrap();
        handle.join().unwrap();

        let file_contents = read_lines_from_file("./src/logger/test_logs/test_1/download_log.txt");
        assert_eq!(file_contents.len(), 1);
        assert_eq!(file_contents[0], "Received piece: 1");
    }

    #[test]
    fn when_logging_5_pieces_then_all_logs_exists_in_file() {
        let path = String::from("./src/logger/test_logs/test_2");
        let (logger, mut worker) = Logger::new(&path).unwrap();

        let handle = std::thread::spawn(move || {
            let _ = worker.listen();
        });

        let _ = logger.log_piece(1).unwrap();
        let _ = logger.log_piece(2).unwrap();
        let _ = logger.log_piece(3).unwrap();
        let _ = logger.log_piece(4).unwrap();
        let _ = logger.log_piece(5).unwrap();

        logger.stop_logging().unwrap();
        handle.join().unwrap();

        let file_contents = read_lines_from_file("./src/logger/test_logs/test_2/download_log.txt");
        assert_eq!(file_contents.len(), 5);
        assert_eq!(file_contents[0], "Received piece: 1");
        assert_eq!(file_contents[1], "Received piece: 2");
        assert_eq!(file_contents[2], "Received piece: 3");
        assert_eq!(file_contents[3], "Received piece: 4");
        assert_eq!(file_contents[4], "Received piece: 5");
    }

    #[test]
    fn using_logger_without_start_listening_throws_inexistent_listener() {
        let path = String::from("./src/logger/test_logs/test_3");
        let (logger, _) = Logger::new(&path).unwrap();
        let result = logger.log_piece(1);
        assert!(matches!(result, Err(LoggerError::InexistentListener)));
    }
}
