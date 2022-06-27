use super::ClientInfo;
use crate::application_errors::ApplicationError;
use crate::peer::Peer;
use crate::peer_connection_manager::*;
use crate::piece_manager::*;
use crate::piece_saver::*;
use crate::ui::UIMessageSender;
use log::*;
use std::thread::JoinHandle;

pub struct ClientHandles {
    piece_manager: JoinHandle<()>,
    piece_saver: JoinHandle<()>,
    peer_connection_manager: JoinHandle<()>,
}

struct ClientSenders {
    pub piece_manager: PieceManagerSender,
    pub piece_saver: PieceSaverSender,
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
                piece_manager_sender.clone(),
                piece_saver_sender.clone(),
                client_info,
                ui_message_sender,
            );

        Ok(TorrentClient {
            senders: ClientSenders {
                piece_manager: piece_manager_sender,
                piece_saver: piece_saver_sender,
                peer_connection_manager: peer_connection_manager_sender,
            },
            workers: ClientWorkers {
                piece_manager: piece_manager_worker,
                piece_saver: piece_saver_worker,
                peer_connection_manager: peer_connection_manager_worker,
            },
        })
    }

    pub fn run_with_peers(mut self, peer_list: Vec<Peer>) -> Result<(), ApplicationError> {
        self.senders
            .piece_manager
            .start(self.senders.peer_connection_manager.clone());

        let piece_saver_handle = std::thread::spawn(move || {
            self.workers.piece_saver.listen().unwrap();
        });

        let piece_manager_handle = std::thread::spawn(move || {
            let _ = self
                .workers
                .piece_manager
                .listen(self.senders.peer_connection_manager);
        });

        let peer_connection_manager_handle = std::thread::spawn(move || {
            self.workers
                .peer_connection_manager
                .start_peer_connections(&peer_list);
            self.workers.peer_connection_manager.listen().unwrap();
        });

        info!("Workers started running");

        let handles = ClientHandles {
            piece_manager: piece_manager_handle,
            piece_saver: piece_saver_handle,
            peer_connection_manager: peer_connection_manager_handle,
        };

        Self::wait_to_end(
            self.senders.piece_manager,
            self.senders.piece_saver,
            handles,
        )?;

        Ok(())
    }

    fn wait_to_end(
        piece_manager: PieceManagerSender,
        piece_saver: PieceSaverSender,
        handles: ClientHandles,
    ) -> Result<(), ApplicationError> {
        handles.piece_manager.join()?;
        piece_manager.stop();
        piece_saver.stop();
        handles.piece_saver.join()?;
        handles.peer_connection_manager.join()?;

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
        new_piece_saver(
            piece_manager_sender,
            client_info.metainfo.info.pieces.clone(),
            client_info.config.download_path.clone(),
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
