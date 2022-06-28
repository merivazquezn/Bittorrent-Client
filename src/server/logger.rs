use super::constants::*;
use crate::logger::LoggerError;
use crate::logger::*;
use crate::peer::PeerMessage;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread::JoinHandle;

enum ServerLoggerMessage {
    BlockSuccesfullySent(usize, usize), // piece_index, block_number
    BlockFailedToSend(usize, usize),    // piece_index, block_number
    ReceivedMessage(PeerMessage),
    ClientDoesntHavePiece(usize),
    Stop,
}

#[derive(Clone)]
/// Struct representing the logger the server will use
pub struct ServerLogger {
    sender: Sender<ServerLoggerMessage>,
}

impl ServerLogger {
    /// Creates a new logger for the server.
    /// The logger starts running and listening inmediatly after created
    /// # Arguments
    /// * `dir_path` - The directory where the logs will be saved.
    ///
    /// # Returns
    ///
    /// ## On success
    /// A tuple with the logger sender and the logger thread handle.
    ///
    /// ## On error
    /// A `LoggerError`
    ///
    pub fn new(dir_path: &str) -> Result<(Self, JoinHandle<()>), LoggerError> {
        let (tx, rx) = mpsc::channel();
        let file: File = create_log_file_in_dir(SERVER_LOG_FILE_NAME, dir_path)?;
        let builder = std::thread::Builder::new().name("server logger worker".to_string());
        let handle = builder
            .spawn(move || {
                let _ = Self::listen(rx, file);
            })
            .map_err(|_| {
                LoggerError::WorkerCreationError(String::from("Logger failed creating the worker"))
            })?;

        Ok((Self { sender: tx }, handle))
    }

    /// Logs a message showing the message the server received from other peer
    pub fn received_message(&self, message: PeerMessage) -> Result<(), LoggerError> {
        self.sender
            .send(ServerLoggerMessage::ReceivedMessage(message))?;
        Ok(())
    }

    /// Logs a message telling that the server succesfully sent a block to other peer
    /// Receives the piece index and the block number
    pub fn block_sent_succesfully(
        &self,
        piece_index: usize,
        block_number: usize,
    ) -> Result<(), LoggerError> {
        self.sender.send(ServerLoggerMessage::BlockSuccesfullySent(
            piece_index,
            block_number,
        ))?;
        Ok(())
    }

    /// Logs a message telling that the server failed to send a block to other peer
    /// Receives the piece index and the block number
    pub fn failed_sending_block(
        &self,
        piece_index: usize,
        block_number: usize,
    ) -> Result<(), LoggerError> {
        self.sender.send(ServerLoggerMessage::BlockFailedToSend(
            piece_index,
            block_number,
        ))?;
        Ok(())
    }

    /// Logs a message telling that the server doesn't have a piece requested from other peer
    /// Receives the index of the requested piece
    pub fn client_doesnt_have_piece(&self, piece_index: usize) -> Result<(), LoggerError> {
        self.sender
            .send(ServerLoggerMessage::ClientDoesntHavePiece(piece_index))?;
        Ok(())
    }

    /// Stops the logger
    pub fn stop(&self) {
        let _ = self.sender.send(ServerLoggerMessage::Stop);
    }

    fn listen(
        receiver: Receiver<ServerLoggerMessage>,
        mut log_file: File,
    ) -> Result<(), RecvError> {
        loop {
            let message: ServerLoggerMessage = receiver.recv()?;
            match message {
                ServerLoggerMessage::ReceivedMessage(message) => {
                    Self::log_message(&mut log_file, message);
                }
                ServerLoggerMessage::BlockSuccesfullySent(piece_index, block_number) => {
                    Self::log_block_sent_succesfully(&mut log_file, piece_index, block_number);
                }
                ServerLoggerMessage::BlockFailedToSend(piece_index, block_number) => {
                    Self::log_failed_sending_block(&mut log_file, piece_index, block_number);
                }
                ServerLoggerMessage::ClientDoesntHavePiece(piece_index) => {
                    Self::log_client_doesnt_have_piece(&mut log_file, piece_index);
                }
                ServerLoggerMessage::Stop => break,
            }
        }
        Ok(())
    }

    fn log_message(file: &mut File, message: PeerMessage) {
        let _ =
            file.write_all(format!("Received the following message: {:?}\n", message).as_bytes());
    }

    fn log_block_sent_succesfully(file: &mut File, piece_index: usize, block_number: usize) {
        let _ = file.write_all(
            format!(
                "Block {} of piece {} succesfully sent\n",
                block_number, piece_index
            )
            .as_bytes(),
        );
    }

    fn log_failed_sending_block(file: &mut File, piece_index: usize, block_number: usize) {
        let _ = file.write_all(
            format!(
                "Block {} of piece {} failed to send\n",
                block_number, piece_index
            )
            .as_bytes(),
        );
    }

    fn log_client_doesnt_have_piece(file: &mut File, piece_index: usize) {
        let _ = file.write_all(format!("Client doesn't have piece {}\n", piece_index).as_bytes());
    }
}
