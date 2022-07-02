use crate::peer::Bitfield;
use crate::peer_connection_manager::PeerConnectionManagerSender;
use crate::piece_manager::types::PieceManagerMessage;
use crate::ui::UIMessageSender;
use log::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvError;

type PeerId = Vec<u8>;
const FIRST_MIN_CONNECTIONS: usize = 100;
pub struct PieceManagerWorker {
    pub reciever: Receiver<PieceManagerMessage>,
    pub allowed_peers_to_download_piece: HashMap<u32, Vec<PeerId>>,
    pub pieces_downloading: HashSet<u32>,
    pub ready_to_download_pieces: HashSet<u32>,
    pub ui_message_sender: UIMessageSender,
    pub is_downloading: bool,
    pub piece_asked_to: HashMap<u32, PeerId>,
    pub pieces_without_peer: HashSet<u32>,
    pub peer_pieces_to_download_count: HashMap<PeerId, u32>,
}

impl PieceManagerWorker {
    fn piece_succesfully_downloaded(&mut self, piece_index: u32, peerd_id: PeerId) {
        self.ready_to_download_pieces.remove(&piece_index);
        self.allowed_peers_to_download_piece.remove(&piece_index);
        self.piece_asked_to.remove(&piece_index);
        let count = self
            .peer_pieces_to_download_count
            .get_mut(&peerd_id)
            .unwrap();
        *count -= 1;
    }

    fn piece_failed_download(
        &mut self,
        piece_index: u32,
        peer_id: PeerId,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
    ) {
        self.ready_to_download_pieces.insert(piece_index);
        self.piece_asked_to.remove(&piece_index);
        self.ask_for_piece(peer_connection_manager_sender);
        let count = self
            .peer_pieces_to_download_count
            .get_mut(&peer_id)
            .unwrap();
        *count -= 1;
    }

    fn last_piece_downloaded(&self) -> bool {
        self.allowed_peers_to_download_piece.is_empty()
    }

    fn update_peers_per_piece(&mut self, bitfield: &Bitfield, peer_id: Vec<u8>) {
        self.allowed_peers_to_download_piece
            .iter_mut()
            .for_each(|(piece_number, peer_ids)| {
                if bitfield.has_piece(*piece_number as usize) {
                    peer_ids.push(peer_id.clone());
                    self.peer_pieces_to_download_count
                        .entry(peer_id.clone())
                        .or_insert(0);
                }
            });
    }

    fn get_optimal_piece_to_download(&self) -> Option<u32> {
        let mut piece_with_less_peers_available_index = None;
        let mut max_peers = 50;
        for (piece_index, peer_ids) in &self.allowed_peers_to_download_piece {
            let peers_of_piece = self
                .allowed_peers_to_download_piece
                .get(piece_index)
                .unwrap();

            if self.ready_to_download_pieces.contains(piece_index)
                && !peers_of_piece.is_empty()
                && peer_ids.len() < max_peers
            {
                max_peers = peer_ids.len();
                piece_with_less_peers_available_index = Some(*piece_index);
            }
        }

        piece_with_less_peers_available_index
    }

    fn ask_for_piece(&mut self, peer_connection_manager_sender: &PeerConnectionManagerSender) {
        let piece = self.get_optimal_piece_to_download();

        if piece.is_none() {
            warn!("no pieces available to download");
            return;
        }

        let piece = piece.unwrap();

        let peers_of_piece = self.allowed_peers_to_download_piece.get(&piece).unwrap();

        let mut peer_id_of_less_pieces_to_download = peers_of_piece[0].clone();
        for peer in peers_of_piece {
            if self.peer_pieces_to_download_count[&peer.clone()]
                < self.peer_pieces_to_download_count[&peer_id_of_less_pieces_to_download]
            {
                peer_id_of_less_pieces_to_download = peer.clone();
            }
        }

        self.ready_to_download_pieces.remove(&piece);
        self.piece_asked_to
            .insert(piece, peer_id_of_less_pieces_to_download.clone());
        if self.pieces_without_peer.contains(&piece) {
            self.pieces_without_peer.remove(&piece);
        }

        let count = self
            .peer_pieces_to_download_count
            .get_mut(&peer_id_of_less_pieces_to_download)
            .unwrap();
        *count += 1;

        peer_connection_manager_sender.download_piece(peer_id_of_less_pieces_to_download, piece);
    }

    fn connection_failed(
        &mut self,
        peer_id: PeerId,
        peer_connection_manager_sender: PeerConnectionManagerSender,
    ) {
        self.allowed_peers_to_download_piece
            .iter_mut()
            .for_each(|(piece, peer_ids)| {
                peer_ids.retain(|x| x != &peer_id);
                if peer_ids.is_empty() && self.pieces_without_peer.contains(piece) {
                    self.pieces_without_peer.insert(*piece);
                }
            });

        let cloned = self.piece_asked_to.clone();
        for (piece, peer) in cloned {
            if peer == peer_id {
                self.piece_asked_to.remove(&piece);
                self.ready_to_download_pieces.insert(piece);
                self.ask_for_piece(&peer_connection_manager_sender);
            }
        }

        self.peer_pieces_to_download_count.remove(&peer_id);
    }

    fn received_have(
        &mut self,
        peer_id: PeerId,
        piece_number: u32,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
    ) {
        if self
            .allowed_peers_to_download_piece
            .contains_key(&piece_number)
        {
            let mut vec = self.allowed_peers_to_download_piece[&piece_number].clone();
            vec.push(peer_id);
            self.allowed_peers_to_download_piece
                .insert(piece_number, vec);

            if self.is_downloading
                && !self.pieces_downloading.contains(&piece_number)
                && self.pieces_without_peer.contains(&piece_number)
            {
                trace!("Asking for piece {} after have msg", piece_number);
                self.ask_for_piece(peer_connection_manager_sender)
            }
        }
    }

    fn ask_for_first_pieces(
        &mut self,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
    ) {
        let aux = self.allowed_peers_to_download_piece.clone();
        let downloadable_first_pieces = aux
            .iter()
            .take_while(|(_, peer_ids)| !peer_ids.is_empty())
            .take(FIRST_MIN_CONNECTIONS);
        downloadable_first_pieces.for_each(|(_, _)| {
            self.ask_for_piece(peer_connection_manager_sender);
        });
    }

    /// Starts downloading, begins with the first FIRST_MIN_CONNECTIONS pieces.
    fn start_downloading(&mut self, peer_connection_manager_sender: &PeerConnectionManagerSender) {
        self.is_downloading = true;
        self.ask_for_first_pieces(peer_connection_manager_sender);
    }

    fn ask_for_pieces_without_peers(
        &mut self,
        peer_connection_manager_sender: &PeerConnectionManagerSender,
    ) {
        let pieces = self.pieces_without_peer.clone();
        pieces.iter().for_each(|piece_number| {
            if self
                .allowed_peers_to_download_piece
                .contains_key(piece_number)
            {
                self.ask_for_piece(peer_connection_manager_sender);
            }
        });
    }

    pub fn listen(
        &mut self,
        peer_connection_manager_sender: PeerConnectionManagerSender,
        _initial_pieces: Vec<u32>,
    ) -> Result<(), RecvError> {
        loop {
            let message = self.reciever.recv()?;
            trace!("Piece manager received message: {:?}", message);
            match message {
                // PieceManagerMessage::Stop => break,
                PieceManagerMessage::Init(_) => {
                    continue;
                }
                PieceManagerMessage::PeerPieces(peer_id, bitfield) => {
                    info!("Received bitfield");
                    trace!("Piece manager received bitfield from peer: {:?}", peer_id);
                    self.update_peers_per_piece(&bitfield, peer_id.clone());
                    trace!("Updated PieceManager hashmap with peer: {:?}", peer_id);
                }
                PieceManagerMessage::FinishedStablishingConnections() => {
                    info!("Piece manager received finished stablishing connections");
                    self.start_downloading(&peer_connection_manager_sender);
                    trace!("Piece manager asked for all pieces");
                }
                PieceManagerMessage::Have(peer_id, piece_number) => {
                    info!("Piece manager received have msg");
                    trace!("Piece manager received Have msg from peer: {:?}", peer_id);
                    self.received_have(peer_id, piece_number, &peer_connection_manager_sender);
                    trace!(
                        "Piece manager updated hashmap with peer having: {:?}",
                        piece_number
                    );
                }
                PieceManagerMessage::ReaskedTracker() => {
                    info!("Piece manager received reasked tracker");
                    self.ask_for_pieces_without_peers(&peer_connection_manager_sender);
                    trace!("Piece manager asked for all pieces");
                }
                PieceManagerMessage::SuccessfulDownload(piece_index, peer_id) => {
                    info!("Piece manager received successful download msg");
                    trace!(
                        "Piece manager received successful download of piece: {:?}",
                        piece_index
                    );
                    self.piece_succesfully_downloaded(piece_index, peer_id);
                    trace!(
                        "Piece manager updated with piece successfully downloaded: {:?}",
                        piece_index
                    );
                }
                PieceManagerMessage::FailedDownload(piece_index, peer_id) => {
                    info!("Piece manager received failed download msg");
                    trace!(
                        "Piece manager received failed download of piece: {}",
                        piece_index
                    );
                    self.piece_failed_download(
                        piece_index,
                        peer_id,
                        &peer_connection_manager_sender.clone(),
                    );
                    trace!(
                        "Piece manager updated with piece failed download: {:?}",
                        piece_index
                    );
                }
                PieceManagerMessage::FailedConnection(peer_id) => {
                    info!(
                        "Piece manager received failed connection with: {:?}",
                        peer_id
                    );
                    self.connection_failed(peer_id.clone(), peer_connection_manager_sender.clone());
                    trace!(
                        "Piece manager updated with failed connection: {:?}",
                        peer_id
                    );
                }
                PieceManagerMessage::FirstConnectionsStarted() => {}
            }
            if self.last_piece_downloaded() {
                info!("Piece manager finished downloading");
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
    use rand::Rng;

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
