use crate::aggregator::AggregatorSender;

use super::announce_manager_sender::AnnounceManager;
use super::announce_manager_worker::AnnounceManagerWorker;

pub fn new_announce_manager(
    aggregator_sender: AggregatorSender,
) -> (AnnounceManager, AnnounceManagerWorker) {
    let (sender, receiver) = std::sync::mpsc::channel();
    (
        AnnounceManager::new(sender),
        AnnounceManagerWorker::new(receiver, aggregator_sender),
    )
}
