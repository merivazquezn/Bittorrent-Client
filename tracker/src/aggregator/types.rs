use crate::aggregator::sender::AggregatorSender;
use crate::aggregator::timer::Timer;
use crate::aggregator::worker::AggregatorWorker;
use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Instant;

use super::AggregatorError;

pub enum AggregatorMessage {
    Increment(String),
    Set(String, i32),
    MinutePassed,
    Stop,
}

pub struct Aggregator {
    pub sender: AggregatorSender,
    pub worker: AggregatorWorker,
    pub timer_handle: std::thread::JoinHandle<()>,
}

impl Aggregator {
    pub fn start() -> Result<Self, AggregatorError> {
        let (tx, rx) = mpsc::channel();

        let aggregator_sender = AggregatorSender { sender: tx };
        let aggregator_sender_cloned = aggregator_sender.clone();

        let timer = Timer::new();
        let timer_handle = std::thread::spawn(move || {
            // if let Err(err) = timer.worker.start(aggregator_sender) {
            // println!("Error in timer: {:?}", err);
            // }
            // habria que ver si se puede sacar el unwrap
            timer.worker.start(aggregator_sender).unwrap();
        });

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
