use crate::aggregator::sender::AggregatorSender;
use crate::aggregator::worker::AggregatorWorker;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use super::{AggregatorError, TimerSender};

pub enum AggregatorMessage {
    Increment(String),
    Set(String, i32),
    MinutePassed,
    Stop,
}

pub struct Aggregator {
    pub sender: AggregatorSender,
    pub worker: AggregatorWorker,
}

impl Aggregator {
    pub fn start(timer_sender: TimerSender) -> Result<Self, AggregatorError> {
        let (tx, rx) = mpsc::channel();

        let aggregator_sender = AggregatorSender { sender: tx };

        Ok(Aggregator {
            sender: aggregator_sender,
            worker: AggregatorWorker {
                receiver: rx,
                aggregation: HashMap::new(),
                last_metrics_update: Instant::now(),
                timer_sender,
            },
        })
    }
}
