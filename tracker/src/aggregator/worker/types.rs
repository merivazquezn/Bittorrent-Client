use crate::aggregator::errors::AggregatorError;
use crate::aggregator::timer::sender::TimerSender;
use crate::aggregator::types::AggregatorMessage;
use crate::metrics::sender::MetricsSender;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::time::Instant;

use crate::aggregator::constants::UPDATE_INTERVAL_SECONDS;

pub struct AggregatorWorker {
    pub receiver: Receiver<AggregatorMessage>,
    pub aggregation: HashMap<String, i32>,
    pub last_metrics_update: Instant,
    pub timer_sender: TimerSender,
}

impl AggregatorWorker {
    fn increment(&mut self, key: String) {
        let amount = self.aggregation.entry(key).or_insert(0);
        *amount += 1;
    }

    fn set(&mut self, key: String, value: i32) {
        let amount = self.aggregation.entry(key).or_insert(0);
        *amount = value;
    }

    fn metrics_update(&mut self, metrics_sender: &MetricsSender) {
        metrics_sender.update(self.aggregation.clone(), Local::now());
    }

    fn metrics_update_if_needed(&mut self, metrics_sender: &MetricsSender) {
        let now = Instant::now();
        let duration = now.duration_since(self.last_metrics_update);
        if duration.as_secs() >= UPDATE_INTERVAL_SECONDS {
            self.metrics_update(metrics_sender);
            self.last_metrics_update = now;
        }
    }

    pub fn listen(&mut self, metrics_sender: MetricsSender) -> Result<(), AggregatorError> {
        loop {
            let message = self.receiver.recv();
            match message {
                Ok(AggregatorMessage::Increment(key)) => {
                    self.metrics_update_if_needed(&metrics_sender);
                    self.increment(key)
                }
                Ok(AggregatorMessage::Set(key, value)) => {
                    self.metrics_update_if_needed(&metrics_sender);
                    self.set(key, value)
                }
                Ok(AggregatorMessage::MinutePassed) => {
                    self.metrics_update_if_needed(&metrics_sender);
                }
                Ok(AggregatorMessage::Stop) => {
                    self.timer_sender.stop();
                    break;
                }
                Err(err) => {
                    return Err(AggregatorError::from(err));
                }
            }
        }
        Ok(())
    }
}
