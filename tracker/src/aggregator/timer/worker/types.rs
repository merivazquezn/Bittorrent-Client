use crate::aggregator::sender::AggregatorSender;
use crate::aggregator::timer::errors::TimerError;
use crate::aggregator::timer::types::TimerMessage;
use crate::aggregator::UPDATE_INTERVAL_SECONDS;
use crate::server::announce::AnnounceManager;
use log::*;
use std::result::Result::Err;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError::Timeout;
use std::time::Duration;

pub struct TimerWorker {
    pub receiver: Receiver<TimerMessage>,
}

impl TimerWorker {
    pub fn start(
        &self,
        aggregator_sender: AggregatorSender,
        announce_manager_sender: AnnounceManager,
    ) -> Result<(), TimerError> {
        let d = Duration::from_secs(UPDATE_INTERVAL_SECONDS);
        loop {
            match self.receiver.recv_timeout(d) {
                Ok(TimerMessage::Stop) => {
                    info!("TimerWorker: Stopping");
                    break;
                }
                Err(err) => {
                    if err == Timeout {
                        info!("TimerWorker: Minute passed");
                        announce_manager_sender.update();
                        aggregator_sender.minute_passed();
                    } else {
                        error!("TimerWorker: Error: {}", err);
                        return Err(TimerError::from(err));
                    }
                }
            }
        }
        Ok(())
    }
}
