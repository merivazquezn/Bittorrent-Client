use std::sync::mpsc::{self, RecvError};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use crate::piece_manager::PieceManager;

#[allow(dead_code)]
pub enum PieceSaverMessage {
    DummyMessage,
    OtherMessage,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct PieceSaver {
    sender: Sender<PieceSaverMessage>,
    piece_manager: PieceManager,
}

impl PieceSaver {
    pub fn new(piece_manager: PieceManager) -> (Self, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            Self::listen(rx).unwrap();
        });

        (
            Self {
                sender: tx,
                piece_manager,
            },
            handle,
        )
    }

    pub fn stop(&self) {
        let _ = self.sender.send(PieceSaverMessage::DummyMessage);
    }

    fn listen(receiver: Receiver<PieceSaverMessage>) -> Result<(), RecvError> {
        loop {
            let message = receiver.recv()?;
            match message {
                PieceSaverMessage::DummyMessage => break,
                PieceSaverMessage::OtherMessage => {
                    unreachable!()
                }
            }
        }

        Ok(())
    }
}
