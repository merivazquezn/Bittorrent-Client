use crate::logger::CustomLogger;
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
const LOGGER: CustomLogger = CustomLogger::init("Piece Manager");

pub struct PieceManagerWorker {
    pub reciever: Receiver<PieceManagerMessage>,
    pub bitfields: HashMap<Vec<u8>, Bitfield>,
    // key: piece number, value: peer_ids
    // each piece number is associated with a vector of the peer_ids that have it
    pub peers_per_piece: HashMap<u32, Vec<Vec<u8>>>,
    pub remaining_pieces: HashSet<u32>,
    pub pieces_downloading: HashSet<u32>,
    pub ui_message_sender: UIMessageSender,
    pub is_downloading: bool,
}

impl PieceManagerWorker {
    /// Updates the state after a pieces sownloading success.
    /// Removes the pieces of the remaining and downloading sets.
    /// And removes it from the peers_per_piece HashMap.
    fn piece_succesfully_downloaded(&mut self, piece_index: u32) {
        self.remaining_pieces.remove(&piece_index);
        self.pieces_downloading.remove(&piece_index);
        self.peers_per_piece.remove(&piece_index);
    }

    /// Updates the state after a piece downloading failure
    /// Removes the piece of the pieces_downloading HashSet and reinserts the piece in the remaining_pieces HashSet
    fn piece_failed_download(&mut self, piece_index: u32) {
        self.pieces_downloading.remove(&piece_index);
        self.remaining_pieces.insert(piece_index);
    }

    /// Returns true if there are no longer any pieces remaining to download nor downloading.
    fn last_piece_downloaded(&self) -> bool {
        self.remaining_pieces.is_empty() && self.pieces_downloading.is_empty()
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

    /// Returns true if all pieces in the peers_per_piece HashMap have at least one peer to download from.
    fn ready_to_download_file(&self) -> bool {
        self.peers_per_piece
            .iter()
            .all(|(_, peer_ids)| !peer_ids.is_empty())
    }

    /// For each piece of the file sends to peer_connection_manager one peer whom to download it from.
    fn ask_for_pieces(&mut self, peer_connection_manager_sender: &PeerConnectionManagerSender) {
        trace!("Asking for pieces");
        self.peers_per_piece
            .iter_mut()
            .for_each(|(piece_number, peer_ids)| {
                peer_connection_manager_sender.download_piece(peer_ids[0].clone(), *piece_number);
            });
    }

    /// Sends the peer_connection_manager the information to download a piece from X peer.
    /// Should discuss whether to aply some logic for this to be more efficient. Function should be called
    /// when a download failed, therefore we need to retry downloading the piece.
    /// Current logic: Randomly selects a peer from the peers_per_piece HashMap.
    fn ask_for_piece(
        &mut self,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
        piece_number: u32,
    ) {
        let len = self.peers_per_piece[&piece_number].len();
        let random_idx = rand::thread_rng().gen_range(0..len);
        let peer_id = self.peers_per_piece[&piece_number][random_idx].clone();
        peer_connection_manager_sender.download_piece(peer_id, piece_number);
    }

    /// Updates the state after receiving a peers bitfield.
    /// Updates the peers_per_pieces Hashmap and inserts the bitfield into our bitfields vector.
    fn receiving_peer_pieces(&mut self, peer_id: Vec<u8>, bitfield: Bitfield) {
        self.bitfields.insert(peer_id.clone(), bitfield.clone());
        self.update_peers_per_piece(&bitfield, peer_id);
    }

    pub fn listen(
        &mut self,
        peer_connection_manager_sender: PeerConnectionManagerSender,
    ) -> Result<(), RecvError> {
        loop {
            let message = self.reciever.recv()?;
            trace!("Piece manager received message: {:?}", message);
            match message {
                PieceManagerMessage::Stop => break,
                PieceManagerMessage::Init(_) => {
                    continue;
                }
                PieceManagerMessage::PeerPieces(peer_id, bitfield) => {
                    trace!("Piece manager Received bitfield from peer: {:?}", peer_id);
                    self.receiving_peer_pieces(peer_id, bitfield);
                    if !self.is_downloading && self.ready_to_download_file() {
                        self.ask_for_pieces(&peer_connection_manager_sender);
                        self.is_downloading = true;
                        LOGGER.info_str(
                            "All pieces can be downloaded, Piece manager starting download",
                        );
                    }
                }
                PieceManagerMessage::SuccessfulDownload(piece_index) => {
                    self.piece_succesfully_downloaded(piece_index);
                    self.ui_message_sender.send_downloaded_piece();
                }
                PieceManagerMessage::FailedDownload(piece_index) => {
                    trace!("failed download of piece: {}", piece_index);
                    self.piece_failed_download(piece_index);
                    self.ask_for_piece(&peer_connection_manager_sender, piece_index);
                }
            }
            if self.last_piece_downloaded() {
                peer_connection_manager_sender.close_connections();
                break;
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
    }
}
