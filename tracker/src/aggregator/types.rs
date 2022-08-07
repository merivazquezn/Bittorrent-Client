use crate::aggregator::sender::AggregatorSender;
use crate::aggregator::timer::Timer;
use crate::aggregator::worker::AggregatorWorker;
use std::collections::HashMap;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Instant;

use super::{AggregatorError, TimerError};

pub enum AggregatorMessage {
    Increment(String),
    Set(String, i32),
    MinutePassed,
    Stop,
}

pub struct Aggregator {
    pub sender: AggregatorSender,
    pub worker: AggregatorWorker,
    pub timer_handle: std::thread::JoinHandle<Result<(), TimerError>>,
}

impl Aggregator {
    pub fn start() -> Result<Self, AggregatorError> {
        let (tx, rx) = mpsc::channel();

        let aggregator_sender = AggregatorSender { sender: tx };
        let aggregator_sender_cloned = aggregator_sender.clone();

        let timer = Timer::new();
        let timer_handle: JoinHandle<Result<(), TimerError>> =
            std::thread::spawn(move || timer.worker.start(aggregator_sender));

        Ok(Aggregator {
            sender: aggregator_sender_cloned,
            worker: AggregatorWorker {
                receiver: rx,
                aggregation: HashMap::new(),
                last_metrics_update: Instant::now(),
                timer_sender: timer.sender,
            },
            timer_handle,
        })
    }
}
