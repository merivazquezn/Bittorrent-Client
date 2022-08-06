use crate::aggregator::types::AggregatorMessage;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct AggregatorSender {
    pub sender: Sender<AggregatorMessage>,
}

impl AggregatorSender {
    pub fn increment(&self, key: String) {
        let _ = self.sender.send(AggregatorMessage::Increment(key));
    }

    pub fn set(&self, key: String, value: i32) {
        let _ = self.sender.send(AggregatorMessage::Set(key, value));
    }

    pub fn minute_passed(&self) {
        let _ = self.sender.send(AggregatorMessage::MinutePassed);
    }
}
