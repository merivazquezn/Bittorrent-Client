use super::super::types::MetricsMessage;
use crate::metrics::params::*;
use crate::http::IHttpService;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct MetricsSender {
    pub sender: Sender<MetricsMessage>,
}

impl MetricsSender {
    pub fn send_metric(
        &self,
        https_service: Box<dyn IHttpService>,
        metric_key: String,
        time_frame: TimeFrame,
        groupby: GroupBy,
    ) {
        let _ = self.sender.send(MetricsMessage::SendMetric(
            https_service,
            metric_key,
            time_frame,
            groupby,
        ));
    }

    pub fn update(&self, aggregation: HashMap<String, i32>, timestamp: DateTime<Local>) {
        let _ = self
            .sender
            .send(MetricsMessage::Update(aggregation, timestamp));
    }
}
