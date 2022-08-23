use super::sender::*;
use super::worker::*;
use crate::metrics::params::*;
use chrono::prelude::*;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::mpsc::Sender;

pub enum MetricsMessage {
    SendMetric(Sender<String>, String, TimeFrame, GroupBy),
    GetTorrents(Sender<String>),
    Update(HashMap<String, i32>, DateTime<Local>),
    Stop,
}

pub fn new_metrics(
    store_days: u32,
    should_recover_from_dump: bool,
) -> (MetricsSender, MetricsWorker) {
    let (tx, rx) = mpsc::channel();
    (
        MetricsSender { sender: tx },
        MetricsWorker {
            receiver: rx,
            record: HashMap::new(),
            store_minutes: (store_days * 24 * 60) as usize,
            should_recover_from_dump,
        },
    )
}
