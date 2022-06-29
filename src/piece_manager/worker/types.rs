use crate::peer::Bitfield;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::piece_manager::types::PieceManagerMessage;
use crate::ui::UIMessageSender;
use log::*;
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

type PeerId = Vec<u8>;

const FIRST_MIN_CONNECTIONS: usize = 5;

pub struct PieceManagerWorker {
    pub reciever: Receiver<PieceManagerMessage>,
    pub peers_per_piece: HashMap<u32, Vec<PeerId>>,
    pub pieces_downloading: HashSet<u32>,
    pub ui_message_sender: UIMessageSender,
    pub is_downloading: bool,
}

impl PieceManagerWorker {
    /// Updates the state after a pieces sownloading success.
    /// Removes the pieces of the downloading set.
    /// And removes it from the peers_per_piece HashMap.
    /// Informs the UI about the downloaded piece.
    fn piece_succesfully_downloaded(&mut self, piece_index: u32) {
        self.pieces_downloading.remove(&piece_index);
        self.peers_per_piece.remove(&piece_index);
        self.ui_message_sender.send_downloaded_piece();
    }

    /// Updates the state after a piece downloading failure
    /// Removes the piece of the pieces_downloading HashSet
    /// Reasks for the piece
    fn piece_failed_download(
        &mut self,
        piece_index: u32,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
    ) {
        self.pieces_downloading.remove(&piece_index);
        self.ask_for_piece(peer_connection_manager_sender, piece_index);
    }

    /// Returns true if there are no longer any pieces remaining to download nor downloading.
    fn last_piece_downloaded(&self) -> bool {
        self.peers_per_piece.is_empty()
    }

    /// Gets a peers peer_id and bitfield.
    /// Iterates the peer_per_pieces HashMap and adds the peer_id to the vector of peer_ids for each piece in the bitfield.
    fn update_peers_per_piece(&mut self, bitfield: &Bitfield, peer_id: Vec<u8>) {
        self.peers_per_piece
            .iter_mut()
            .for_each(|(piece_number, peer_ids)| {
                if bitfield.has_piece(*piece_number as usize) {
                    peer_ids.push(peer_id.clone());
                }
            });
    }

    /// Sends the peer_connection_manager the information to download a piece from X peer.
    /// Should discuss whether to aply some logic for this to be more efficient. Function should be called
    /// when a download failed, therefore we need to retry downloading the piece.
    /// Current logic: Randomly selects a peer from the peers_per_piece HashMap.
    ///
    /// If there are no peers to give piece, does nothing. Eventually when receiving a have msg we will ask for it.
    /// If piece is already downloading does nothing.
    fn ask_for_piece(
        &mut self,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
        piece_number: u32,
    ) {
        let len = self.peers_per_piece[&piece_number].len();
        if len > 0 && !self.pieces_downloading.contains(&piece_number) {
            let random_idx = rand::thread_rng().gen_range(0..len);
            let peer_id = self.peers_per_piece[&piece_number][random_idx].clone();
            peer_connection_manager_sender.download_piece(peer_id, piece_number);
            self.pieces_downloading.insert(piece_number);
        }
    }

    /// For each piece of the file sends to peer_connection_manager one peer whom to download it from.
    fn ask_for_pieces(&mut self, peer_connection_manager_sender: &PeerConnectionManagerSender) {
        trace!("Asking for pieces");
        let mut aux = self.peers_per_piece.clone();
        aux.iter_mut().for_each(|(piece, _)| {
            self.ask_for_piece(peer_connection_manager_sender, *piece);
        });
    }

    /// Updates the state after receiving a peers bitfield.
    /// Updates the peers_per_pieces Hashmap.
    fn receiving_peer_pieces(&mut self, peer_id: PeerId, bitfield: Bitfield) {
        self.update_peers_per_piece(&bitfield, peer_id);
    }

    /// Removes the peer from the bitfields vector and peers_per_piece hashmap.
    fn connection_failed(&mut self, peer_id: PeerId) {
        self.peers_per_piece.iter_mut().for_each(|(_, peer_ids)| {
            peer_ids.retain(|x| x != &peer_id);
        });
    }

    /// Updates the state after receiving a have message.
    /// If the piece is not downloading already, and the system is downloading, then it asks for the piece.
    fn received_have(
        &mut self,
        peer_id: PeerId,
        piece_number: u32,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
    ) {
        self.peers_per_piece
            .entry(piece_number)
            .or_insert(Vec::new())
            .push(peer_id);
        if !self.pieces_downloading.contains(&piece_number) && self.is_downloading {
            self.ask_for_piece(peer_connection_manager_sender, piece_number)
        }
    }

    /// Asks for the first FIRST_MIN_CONNECTIONS pieces.
    fn ask_for_first_pieces(
        &mut self,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
    ) {
        let aux = self.peers_per_piece.clone();
        let downloadable_first_pieces = aux
            .iter()
            .take_while(|(_, peer_ids)| !peer_ids.is_empty())
            .take(FIRST_MIN_CONNECTIONS);
        downloadable_first_pieces.for_each(|(piece_number, _)| {
            self.ask_for_piece(peer_connection_manager_sender, *piece_number);
        });
    }

    /// Starts downloading, begins with the first FIRST_MIN_CONNECTIONS pieces.
    fn start_downloading(&mut self, peer_connection_manager_sender: &PeerConnectionManagerSender) {
        self.is_downloading = true;
        self.ask_for_first_pieces(peer_connection_manager_sender)
    }

    pub fn listen(
        &mut self,
        peer_connection_manager_sender: PeerConnectionManagerSender,
    ) -> Result<(), RecvError> {
        loop {
            let message = self.reciever.recv()?;
            trace!("Piece manager received message: {:?}", message);
            match message {
                PieceManagerMessage::Init(_) => {
                    continue;
                }
                PieceManagerMessage::PeerPieces(peer_id, bitfield) => {
                    trace!("Piece manager received bitfield from peer: {:?}", peer_id);
                    self.receiving_peer_pieces(peer_id, bitfield);
                }
                PieceManagerMessage::FirstConnectionsStarted() => {
                    self.start_downloading(&peer_connection_manager_sender);
                }
                PieceManagerMessage::FinishedStablishingConnections() => {
                    self.ask_for_pieces(&peer_connection_manager_sender);
                }
                PieceManagerMessage::Have(peer_id, piece_number) => {
                    trace!("Piece manager received Have msg from peer: {:?}", peer_id);
                    self.received_have(peer_id, piece_number, &peer_connection_manager_sender);
                }
                PieceManagerMessage::SuccessfulDownload(piece_index) => {
                    trace!(
                        "Piece manager received successful download of piece: {:?}",
                        piece_index
                    );
                    self.piece_succesfully_downloaded(piece_index);
                    if self.last_piece_downloaded() {
                        peer_connection_manager_sender.close_connections();
                        break;
                    }
                }
                PieceManagerMessage::FailedDownload(piece_index) => {
                    trace!("failed download of piece: {}", piece_index);
                    self.piece_failed_download(piece_index, &peer_connection_manager_sender);
                }
                PieceManagerMessage::FailedConnection(peer_id) => {
                    trace!("failed connection with: {:?}", peer_id);
                    self.connection_failed(peer_id);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn peer_per_piece_updates_verifys_if_ready_and_select_peer_correctly() {
        // in this case the entire file has 5 pieces
        let total_pieces = 5;

        // create peer_per_pieces
        let mut peers_per_piece = HashMap::new();

        // initialize peers_per_piece
        for i in 0..total_pieces {
            let vec: Vec<Vec<u8>> = Vec::new();
            peers_per_piece.insert(i, vec);
        }

        let peer_id_1: Vec<u8> = vec![9, 8, 7];
        let peer_id_2: Vec<u8> = vec![6, 5, 4];
        let peer_id_3: Vec<u8> = vec![9, 5, 4];

        let bf_1 = vec![0 as u8, 1 as u8, 2 as u8];
        let bf_2 = vec![1 as u8, 2 as u8, 4 as u8];
        let bf_3 = vec![0 as u8, 3 as u8, 4 as u8];

        let mut bitfield_1 = Bitfield::new();
        bitfield_1.set_bitfield(&bf_1);

        let mut bitfield_2 = Bitfield::new();
        bitfield_2.set_bitfield(&bf_2);

        let mut bitfield_3 = Bitfield::new();
        bitfield_3.set_bitfield(&bf_3);

        peers_per_piece
            .iter_mut()
            .for_each(|(piece_number, peer_ids)| {
                if *piece_number == 0 {
                    peer_ids.push(peer_id_1.clone());
                    peer_ids.push(peer_id_3.clone());
                } else if *piece_number == 1 || *piece_number == 2 {
                    peer_ids.push(peer_id_1.clone());
                    peer_ids.push(peer_id_2.clone());
                } else if *piece_number == 3 {
                    peer_ids.push(peer_id_3.clone());
                } else if *piece_number == 4 {
                    peer_ids.push(peer_id_2.clone());
                    peer_ids.push(peer_id_3.clone());
                }
            });

        // check if peers_per_piece is updated correctly
        peers_per_piece.iter().for_each(|(piece_number, peer_ids)| {
            if *piece_number == 0 {
                assert_eq!(peer_ids[0], peer_id_1);
                assert_eq!(peer_ids[1], peer_id_3);
            } else if *piece_number == 1 || *piece_number == 2 {
                assert_eq!(peer_ids[0], peer_id_1);
                assert_eq!(peer_ids[1], peer_id_2);
            } else if *piece_number == 4 {
                assert_eq!(peer_ids[0], peer_id_2);
                assert_eq!(peer_ids[1], peer_id_3);
            } else if *piece_number == 3 {
                assert_eq!(peer_ids[0], peer_id_3);
            }
        });

        // check if ready_to_download_file is true, should be, we have all the pieces
        assert_eq!(
            peers_per_piece
                .iter()
                .all(|(_, peer_ids)| !peer_ids.is_empty()),
            true
        );

        let piece_number = 0;
        let len = peers_per_piece[&piece_number].len();
        let random_idx = rand::thread_rng().gen_range(0..len);
        let peer_id = peers_per_piece[&piece_number][random_idx].clone();

        // check if we are asking the correct peer for the piece
        assert_eq!(peer_id == vec![9, 5, 4] || peer_id == vec![9, 8, 7], true);

        // deleting the peer_id_1 from the peers_per_piece
        peers_per_piece.iter_mut().for_each(|(_, peer_ids)| {
            peer_ids.retain(|x| x != &peer_id_1);
        });

        peers_per_piece.iter().for_each(|(piece_number, peer_ids)| {
            if *piece_number == 0 {
                assert_eq!(peer_ids.len(), 1);
                assert_eq!(peer_ids[0], peer_id_3);
            } else if *piece_number == 1 || *piece_number == 2 {
                assert_eq!(peer_ids.len(), 1);
                assert_eq!(peer_ids[0], peer_id_2);
            } else if *piece_number == 4 {
                assert_eq!(peer_ids[0], peer_id_2);
                assert_eq!(peer_ids[1], peer_id_3);
            } else if *piece_number == 3 {
                assert_eq!(peer_ids[0], peer_id_3);
            }
        });
    }
}
