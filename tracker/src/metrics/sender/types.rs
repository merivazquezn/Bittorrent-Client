use super::super::types::MetricsMessage;
use crate::metrics::params::*;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::{RecvError, Sender};

#[derive(Debug, Clone)]
pub struct MetricsSender {
    pub sender: Sender<MetricsMessage>,
}

impl MetricsSender {
    pub fn get_metrics_response(
        &self,
        metric_key: String,
        time_frame: TimeFrame,
        groupby: GroupBy,
    ) -> Result<String, RecvError> {
        let (sender, receiver) = std::sync::mpsc::channel();
        let _ = self.sender.send(MetricsMessage::SendMetric(
            sender, metric_key, time_frame, groupby,
        ));

        let response: String = receiver.recv()?;
        Ok(response)
    }

    pub fn update(&self, aggregation: HashMap<String, i32>, timestamp: DateTime<Local>) {
        let _ = self
            .sender
            .send(MetricsMessage::Update(aggregation, timestamp));
    }
}
