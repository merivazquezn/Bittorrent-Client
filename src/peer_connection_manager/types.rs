use crate::piece_manager::PieceManager;
use crate::piece_saver::PieceSaver;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread::JoinHandle;

#[allow(dead_code)]
pub enum PeerConnectionManagerMessage {
    DummyMessage,
    Init(PieceManager, PieceSaver),
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct PeerConnectionManager {
    sender: Sender<PeerConnectionManagerMessage>,
    piece_manager: PieceManager,
}

impl PeerConnectionManager {
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

    pub fn start(&self, piece_manager: PieceManager, piece_saver: PieceSaver) {
        let _ = self.sender.send(PeerConnectionManagerMessage::Init(
            piece_manager,
            piece_saver,
        ));
    }

    pub fn stop(&self) {
        let _ = self.sender.send(PeerConnectionManagerMessage::DummyMessage);
    }

    fn listen(receiver: Receiver<PeerConnectionManagerMessage>) -> Result<(), RecvError> {
        let init_message = receiver.recv()?;
        let (_piece_manager, _piece_saver) = match init_message {
            PeerConnectionManagerMessage::Init(piece_manager, piece_saver) => {
                (piece_manager, piece_saver)
            }
            _ => unreachable!(),
        };

        loop {
            let message = receiver.recv()?;
            match message {
                PeerConnectionManagerMessage::DummyMessage => break,
                PeerConnectionManagerMessage::Init(_, _) => {
                    unreachable!()
                }
            }
        }

        Ok(())
    }
}
