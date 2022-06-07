use std::sync::mpsc::{self, RecvError};
use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;

use crate::peer_connection_manager::PeerConnectionManager;

pub enum PieceManagerMessage {
    DummyMessage,
    Init(PeerConnectionManager),
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct PieceManager {
    sender: Sender<PieceManagerMessage>,
}

impl PieceManager {
    pub fn new() -> (Self, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            Self::listen(rx).unwrap();
        });

        (Self { sender: tx }, handle)
    }

    pub fn start(&self, peer_connection_manager: PeerConnectionManager) {
        let _ = self
            .sender
            .send(PieceManagerMessage::Init(peer_connection_manager));
    }

    pub fn stop(&self) {
        let _ = self.sender.send(PieceManagerMessage::DummyMessage);
    }

    fn listen(receiver: Receiver<PieceManagerMessage>) -> Result<(), RecvError> {
        let init_message = receiver.recv()?;
        let _peer_connection_manager = match init_message {
            PieceManagerMessage::Init(peer_connection_manager) => peer_connection_manager,
            _ => unreachable!(),
        };

        loop {
            let message = receiver.recv()?;
            match message {
                PieceManagerMessage::DummyMessage => break,
                PieceManagerMessage::Init(_) => {
                    unreachable!()
                }
            }
        }
        Ok(())
    }
}
