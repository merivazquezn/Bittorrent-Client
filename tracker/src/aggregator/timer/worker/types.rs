use crate::aggregator::sender::AggregatorSender;
use crate::aggregator::timer::errors::TimerError;
use crate::aggregator::timer::types::TimerMessage;
use log::*;
use std::result::Result::Err;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError::Timeout;
use std::time::Duration;

pub struct TimerWorker {
    pub receiver: Receiver<TimerMessage>,
}

impl TimerWorker {
    pub fn start(&self, aggregator_sender: AggregatorSender) -> Result<(), TimerError> {
        let d = Duration::from_millis(60000);
        loop {
            match self.receiver.recv_timeout(d) {
                Ok(TimerMessage::Stop) => {
                    info!("TimerWorker: Stopping");
                    break;
                }
                Err(err) => {
                    if err == Timeout {
                        info!("TimerWorker: Minute passed");
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
