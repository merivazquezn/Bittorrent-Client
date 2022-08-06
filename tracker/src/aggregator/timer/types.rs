use crate::aggregator::timer::sender::TimerSender;
use crate::aggregator::timer::worker::TimerWorker;
use std::sync::mpsc;

pub enum TimerMessage {
    Stop,
}

pub struct Timer {
    pub sender: TimerSender,
    pub worker: TimerWorker,
}

// default constructor
impl Timer {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Timer {
            sender: TimerSender { sender: tx },
            worker: TimerWorker { receiver: rx },
        }
    }
}
