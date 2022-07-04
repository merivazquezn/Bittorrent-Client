use super::ClientInfo;
use crate::application_errors::ApplicationError;
use crate::download_manager;
use crate::download_manager::get_existing_pieces;
use crate::peer_connection_manager::*;
use crate::piece_manager::*;
use crate::piece_saver::*;
use crate::tracker::ITrackerService;
use crate::tracker::TrackerResponse;
use crate::ui::UIMessageSender;
use log::*;
use std::thread::JoinHandle;

pub struct ClientHandles {
    piece_manager: JoinHandle<()>,
    piece_saver: JoinHandle<()>,
    peer_connection_manager: JoinHandle<()>,
}

struct ClientSenders {
    pub peer_connection_manager: PeerConnectionManagerSender,
}

struct ClientWorkers {
    pub piece_manager: PieceManagerWorker,
    pub piece_saver: PieceSaverWorker,
    pub peer_connection_manager: PeerConnectionManagerWorker,
}

pub struct TorrentClient {
    senders: ClientSenders,
    workers: ClientWorkers,
}

impl TorrentClient {
    pub fn new(
        client_info: &ClientInfo,
        ui_message_sender: UIMessageSender,
    ) -> Result<Self, ApplicationError> {
        let (piece_manager_sender, piece_manager_worker) =
            Self::init_piece_manager(client_info, ui_message_sender.clone());

        let (piece_saver_sender, piece_saver_worker) = Self::init_piece_saver(
            piece_manager_sender.clone(),
            client_info,
            ui_message_sender.clone(),
        );

        let (peer_connection_manager_sender, peer_connection_manager_worker) =
            Self::init_peer_connection_manager(
                piece_manager_sender,
                piece_saver_sender,
                client_info,
                ui_message_sender,
            );

        Ok(TorrentClient {
            senders: ClientSenders {
                peer_connection_manager: peer_connection_manager_sender,
            },
            workers: ClientWorkers {
                piece_manager: piece_manager_worker,
                piece_saver: piece_saver_worker,
                peer_connection_manager: peer_connection_manager_worker,
            },
        })
    }

    pub fn run(
        mut self,
        client_info: ClientInfo,
        tracker_service: Box<dyn ITrackerService + Send>,
        tracker_response: TrackerResponse,
    ) -> Result<(), ApplicationError> {
        let piece_saver_handle = std::thread::spawn(move || {
            self.workers.piece_saver.listen().unwrap();
        });

        let peer_connection_manager_sender_clone = self.senders.peer_connection_manager.clone();

        let download_path = format!(
            "{}/{}",
            client_info.config.download_path, client_info.metainfo.info.name
        );

        let initial_pieces: Vec<u32> = get_existing_pieces(
            client_info.metainfo.get_piece_count(),
            format!("{}/pieces", download_path).as_str(),
        );
        let piece_manager_handle = std::thread::spawn(move || {
            let _ = self
                .workers
                .piece_manager
                .listen(peer_connection_manager_sender_clone, initial_pieces);
        });

        let peer_connection_manager_sender_clone = self.senders.peer_connection_manager.clone();
        let peer_connection_manager_handle = std::thread::spawn(move || {
            self.workers.peer_connection_manager.start_peer_connections(
                tracker_response.peers,
                peer_connection_manager_sender_clone.clone(),
            );
            self.workers
                .peer_connection_manager
                .listen(
                    tracker_service,
                    tracker_response.interval,
                    peer_connection_manager_sender_clone,
                )
                .unwrap();
        });

        let handles = ClientHandles {
            piece_manager: piece_manager_handle,
            piece_saver: piece_saver_handle,
            peer_connection_manager: peer_connection_manager_handle,
        };

        Self::wait_to_end(handles)?;

        info!("About to join pieces into target file");

        let download_path = format!(
            "{}/{}",
            client_info.config.download_path, client_info.metainfo.info.name
        );

        download_manager::make_target_file(
            client_info.metainfo.get_piece_count(),
            &client_info.metainfo.info.name,
            &download_path,
            client_info.config.persist_pieces,
        )?;

        Ok(())
    }

    fn wait_to_end(handles: ClientHandles) -> Result<(), ApplicationError> {
        handles.piece_manager.join()?;
        info!("Piece manager joined");
        handles.piece_saver.join()?;
        info!("Piece saver joined");

        handles.peer_connection_manager.join()?;
        info!("Peer connection joined");

        info!("All workers stopped running");
        Ok(())
    }

    fn init_piece_manager(
        client_info: &ClientInfo,
        ui_message_sender: UIMessageSender,
    ) -> (PieceManagerSender, PieceManagerWorker) {
        new_piece_manager(
            client_info.metainfo.info.pieces.len() as u32,
            ui_message_sender,
        )
    }

    fn init_piece_saver(
        piece_manager_sender: PieceManagerSender,
        client_info: &ClientInfo,
        ui_message_sender: UIMessageSender,
    ) -> (PieceSaverSender, PieceSaverWorker) {
        let donwload_path = format!(
            "{}/{}",
            client_info.config.download_path, client_info.metainfo.info.name
        );
        new_piece_saver(
            piece_manager_sender,
            client_info.metainfo.info.pieces.clone(),
            donwload_path,
            ui_message_sender,
        )
    }

    fn init_peer_connection_manager(
        piece_manager_sender: PieceManagerSender,
        piece_saver_sender: PieceSaverSender,
        client_info: &ClientInfo,
        ui_message_sender: UIMessageSender,
    ) -> (PeerConnectionManagerSender, PeerConnectionManagerWorker) {
        new_peer_connection_manager(
            piece_manager_sender,
            piece_saver_sender,
            &client_info.metainfo,
            &client_info.peer_id,
            ui_message_sender,
        )
    }
}
