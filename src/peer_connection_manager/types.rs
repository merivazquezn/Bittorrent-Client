use crate::piece_manager::sender::PieceManagerSender;
use crate::piece_saver::sender::PieceSaverSender;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, Sender};
use std::thread::JoinHandle;

#[allow(dead_code)]
pub enum PeerConnectionManagerMessage {
    DownloadPiece(u64, u64),
    Init(PieceManagerSender, PieceSaverSender),
    CloseConnections,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct PeerConnectionManager {
    sender: Sender<PeerConnectionManagerMessage>,
    //PeerConnections: Vec<PeerConnection> or something of the sort
    //each element in this array will inform if the connection is currently open or closed
}

impl PeerConnectionManager {
    pub fn new() -> (Self, JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            Self::listen(rx).unwrap();
        });

        (Self { sender: tx }, handle)
    }

    pub fn start(
        &self,
        piece_manager_sender: PieceManagerSender,
        piece_saver_sender: PieceSaverSender,
    ) {
        let _ = self.sender.send(PeerConnectionManagerMessage::Init(
            piece_manager_sender,
            piece_saver_sender,
        ));
    }

    pub fn stop(&self) {
        let _ = self
            .sender
            .send(PeerConnectionManagerMessage::CloseConnections);
    }

    //Should be used when receiving "CloseConnections" message
    /*fn terminate_connections_and_piece_saver_sender(piece_saver_sender: PieceSaverSender) {
        for connection in self.PeerConnections{
            connection.close();
        }
        piece_saver_sender.stop();
    }*/

    //Should be used when receiving "DownloadPiece" message
    /*fn download_piece(peer_index: u64, piece_index: u64){
        if peer_index > self.PeerConnections.len(){
            return PeerConnectionManagerError:InvalidPeerIndex;
        }
        peer_connection = self.PeerConnections[peer_index]

        if !peer_connection.is_open(){
            peer_connection.open();
        }
        peer_connection.download_piece(piece_index);
    }*/

    fn listen(receiver: Receiver<PeerConnectionManagerMessage>) -> Result<(), RecvError> {
        let init_message = receiver.recv()?;
        let (_piece_manager_sender, _piece_saver_sender) = match init_message {
            PeerConnectionManagerMessage::Init(piece_manager_sender, piece_saver_sender) => {
                (piece_manager_sender, piece_saver_sender)
            }
            _ => unreachable!(),
        };

        loop {
            let message = receiver.recv()?;
            match message {
                PeerConnectionManagerMessage::Init(_, _) => unreachable!(),
                PeerConnectionManagerMessage::CloseConnections => break,
                PeerConnectionManagerMessage::DownloadPiece(_peer_index, _piece_index) => break,
            }
        }

        Ok(())
    }
}
