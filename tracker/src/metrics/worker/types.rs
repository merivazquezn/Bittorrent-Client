use super::super::types::MetricsMessage;
use crate::application_constants::*;
use crate::http::IHttpService;
use crate::metrics::grouping_methods::*;
use crate::metrics::params::*;
use chrono::prelude::*;
use chrono::Duration;
use chrono::DurationRound;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvError};

pub struct MetricsWorker {
    pub receiver: Receiver<MetricsMessage>,
    pub record: HashMap<String, Vec<(i32, DateTime<Local>)>>,
    pub store_minutes: usize,
}

impl MetricsWorker {
    pub fn listen(&mut self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            match message {
                MetricsMessage::SendMetric(https_service, key, time_frame, groupby) => {
                    self.send_metric(https_service, key, time_frame, groupby)
                }
                MetricsMessage::Update(aggregation, timestamp) => {
                    self.update(aggregation, timestamp)
                }
                MetricsMessage::Stop => break,
            }
        }
        Ok(())
    }

    fn get_lower_bound(record_vector: &[(i32, DateTime<Local>)], timeframe: TimeFrame) -> usize {
        let total_minutes = match timeframe {
            TimeFrame::LastDays(n_days) => (n_days * 24 * 60) as usize,
            TimeFrame::LastHours(n_hours) => (n_hours * 60) as usize,
        };

        if record_vector.len() > total_minutes {
            record_vector.len() - total_minutes
        } else {
            0
        }
    }

    fn choose_grouping_method(metric_key: String) -> Box<dyn ChunkAggregator> {
        let stat = if metric_key.contains(KEY_DELIMITER) {
            let split_iter = metric_key.split(KEY_DELIMITER);
            let split: Vec<&str> = split_iter.collect();
            split[1]
        } else {
            &metric_key
        };

        let default = AggregatingMethod::Max;
        match stat {
            ACTIVE_PEERS_STAT => Box::new(AggregatingMethod::Average),
            COMPLETED_DOWNLOADS_STAT => Box::new(AggregatingMethod::Max),
            TORRENTS_STAT => Box::new(AggregatingMethod::Max),
            _ => Box::new(default),
        }
    }

    fn rounded_timestamp_string(timestamp: DateTime<Local>, round_to_duration: Duration) -> String {
        timestamp
            .duration_round(round_to_duration)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
    }

    fn group_slice(
        record_slice: &[(i32, DateTime<Local>)],
        groupby: GroupBy,
        metric_key: String,
    ) -> Vec<(i32, String)> {
        let group_minutes;
        let round_to;
        match groupby {
            GroupBy::Minutes(n_minutes) => {
                group_minutes = n_minutes;
                round_to = Duration::minutes(n_minutes as i64);
            }
            GroupBy::Hours(n_hours) => {
                group_minutes = n_hours * 60;
                round_to = Duration::hours(n_hours as i64);
            }
        }
        let mut grouped = Vec::new();
        let grouping_method = Self::choose_grouping_method(metric_key);
        for chunk in record_slice.chunks(group_minutes as usize) {
            let stat_value = grouping_method.aggregate(chunk);
            let timestamp: String = Self::rounded_timestamp_string(chunk[0].1, round_to);
            grouped.push((stat_value, timestamp));
        }
        grouped
    }

    fn get_json_from_slice(grouped_slice: Vec<(i32, String)>) -> String {
        let mut formatted = Vec::new();
        for (value, timestamp) in grouped_slice {
            let mut map = Map::new();
            map.insert(TIMESTAMP_JSON_KEY.to_string(), json!(timestamp));
            map.insert(VALUE_JSON_KEY.to_string(), json!(value));
            formatted.push(Value::Object(map));
        }

        let mut map = Map::new();
        map.insert(DATA_JSON_KEY.to_string(), json!(formatted));
        let json_object = Value::Object(map);
        json_object.to_string()
    }

    fn send_metric(
        &mut self,
        mut https_service: Box<dyn IHttpService>,
        metric_key: String,
        timeframe: TimeFrame,
        groupby: GroupBy,
    ) {
        if !self.record.contains_key(&metric_key) {
            let _ = https_service
                .send_ok_response("".as_bytes().to_vec(), "application/json".to_string());
            return;
        }

        let record_vector = self.record.get_mut(&metric_key).unwrap();
        let lower_bound = Self::get_lower_bound(record_vector, timeframe);
        let record_slice = &record_vector[lower_bound..];
        let grouped_slice = Self::group_slice(record_slice, groupby, metric_key);

        let json: String = Self::get_json_from_slice(grouped_slice);
        let _ = https_service
            .send_ok_response(json.as_bytes().to_vec(), "application/json".to_string());
    }

    fn update(&mut self, aggregation: HashMap<String, i32>, timestamp: DateTime<Local>) {
        for (key, value) in aggregation.iter() {
            if !self.record.contains_key(key) {
                self.record.insert(key.clone(), vec![(*value, timestamp)]);
                continue;
            }

            let record_vector = self.record.get_mut(key).unwrap();

            record_vector.push((*value, timestamp));

            if record_vector.len() > self.store_minutes {
                record_vector.remove(0);
            }
        }
    }
}
