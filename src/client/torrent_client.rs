use super::ClientInfo;
use crate::application_errors::ApplicationError;
use crate::download_manager;
use crate::peer_connection_manager::*;
use crate::piece_manager::*;
use crate::piece_saver::*;
use crate::tracker::Event;
use crate::tracker::ITrackerService;
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
        initial_pieces: Vec<u32>,
    ) -> Result<Self, ApplicationError> {
        let (piece_manager_sender, piece_manager_worker) =
            Self::init_piece_manager(client_info, ui_message_sender.clone(), initial_pieces);

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
        tracker_service: &mut (impl ITrackerService + Send + 'static),
    ) -> Result<(), ApplicationError> {
        let piece_saver_handle = std::thread::spawn(move || {
            self.workers.piece_saver.listen().unwrap();
        });

        let peer_connection_manager_sender_clone = self.senders.peer_connection_manager.clone();

        let piece_manager_handle = std::thread::spawn(move || {
            let _ = self
                .workers
                .piece_manager
                .listen(peer_connection_manager_sender_clone);
        });

        let tracker_response = tracker_service.announce(Some(Event::Started))?;

        let peer_connection_manager_sender_clone = self.senders.peer_connection_manager.clone();
        let mut tracker_service_clone = tracker_service.clone();
        let peer_connection_manager_handle = std::thread::spawn(move || {
            self.workers.peer_connection_manager.start_peer_connections(
                tracker_response.peers,
                peer_connection_manager_sender_clone.clone(),
            );
            self.workers
                .peer_connection_manager
                .listen(
                    &mut tracker_service_clone,
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

        let target_name = format!(
            "{}/target/{}",
            download_path, client_info.metainfo.info.name
        );

        if !client_info.config.persist_pieces {
            // delete file at target_name
            let _ = std::fs::remove_file(target_name.clone());
        }

        if !std::path::Path::new(&target_name).exists() {
            download_manager::make_target_file(
                client_info.metainfo.get_piece_count(),
                &client_info.metainfo.info.name,
                &download_path,
                client_info.config.persist_pieces,
            )?;

            let _ = tracker_service.announce(Some(Event::Completed));
        }

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
        initial_pieces: Vec<u32>,
    ) -> (PieceManagerSender, PieceManagerWorker) {
        new_piece_manager(
            client_info.metainfo.info.pieces.len() as u32,
            ui_message_sender,
            initial_pieces,
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
