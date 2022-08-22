use crate::aggregator::AggregatorSender;

use super::announce_manager_sender::AnnounceManager;
use super::announce_manager_worker::AnnounceManagerWorker;

/// Creates and returns a new announce manager, both sender and worker
pub fn new_announce_manager(
    aggregator_sender: AggregatorSender,
    interval: u32,
) -> (AnnounceManager, AnnounceManagerWorker) {
    let (sender, receiver) = std::sync::mpsc::channel();
    (
        AnnounceManager::new(sender),
        AnnounceManagerWorker::new(receiver, aggregator_sender, interval),
    )
}
