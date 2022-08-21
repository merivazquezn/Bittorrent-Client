use super::super::types::MetricsMessage;
use crate::application_constants::*;
use crate::metrics::grouping_methods::*;
use crate::metrics::params::*;
use chrono::prelude::*;
use chrono::Duration;
use chrono::DurationRound;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::mpsc::{Receiver, RecvError};

pub const METRIC_NOT_FOUND: &str = "{ \"error\": \"Metric not found\" }";

pub struct MetricsWorker {
    pub receiver: Receiver<MetricsMessage>,
    pub record: HashMap<String, Vec<(i32, DateTime<Local>)>>,
    pub store_minutes: usize,
}

fn timestamp_to_string(timestamp: DateTime<Local>) -> String {
    timestamp
        .duration_round(Duration::minutes(1))
        .unwrap()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

impl MetricsWorker {
    fn send_metric(
        &mut self,
        sender: Sender<String>,
        metric_key: String,
        timeframe: TimeFrame,
        groupby: GroupBy,
    ) {
        if !self.record.contains_key(&metric_key) {
            let error_json: String = METRIC_NOT_FOUND.to_string();
            let _ = sender.send(error_json);
            return;
        }

        let record_vector = self.record.get_mut(&metric_key).unwrap();
        let lower_bound = Self::get_lower_bound(record_vector, timeframe);
        let record_slice = &record_vector[lower_bound..];
        let grouped_slice = Self::group_slice(record_slice, groupby, metric_key);
        let grouped_slice_as_string: Vec<(i32, String)> = grouped_slice
            .iter()
            .map(|tuple| (tuple.0, timestamp_to_string(tuple.1)))
            .collect();

        let json: String = Self::get_json_from_slice(grouped_slice_as_string);
        let _ = sender.send(json);
    }

    fn update(&mut self, aggregation: HashMap<String, i32>, timestamp: DateTime<Local>) {
        println!("hashmap of aggregation: {:?}", aggregation);
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

    pub fn listen(&mut self) -> Result<(), RecvError> {
        loop {
            let message = self.receiver.recv()?;
            match message {
                MetricsMessage::SendMetric(sender, key, time_frame, groupby) => {
                    self.send_metric(sender, key, time_frame, groupby)
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

        let default: AggregatingMethod = AggregatingMethod::Max;
        let method: AggregatingMethod = match stat {
            ACTIVE_PEERS_STAT => AggregatingMethod::Average,
            COMPLETED_DOWNLOADS_STAT => AggregatingMethod::Max,
            TORRENTS_STAT => AggregatingMethod::Max,
            _ => default,
        };

        Box::new(method)
    }

    fn group_slice(
        record_slice: &[(i32, DateTime<Local>)],
        groupby: GroupBy,
        metric_key: String,
    ) -> Vec<(i32, DateTime<Local>)> {
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
            // let timestamp: String = Self::rounded_timestamp_string(chunk[0].1, round_to);
            grouped.push((stat_value, chunk[0].1.duration_round(round_to).unwrap()));
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
}
#[cfg(test)]
mod tests {
    use crate::application_constants::*;
    use crate::metrics::worker::types::METRIC_NOT_FOUND;
    use crate::metrics::*;
    use chrono::prelude::*;
    use std::collections::HashMap;
    use std::thread;

    const STARTING_Y_M_D: (i32, u32, u32) = (2022, 1, 1);

    fn setup(store_days: u32) -> MetricsSender {
        let (metrics_sender, mut metrics_worker) = new_metrics(store_days);
        let _ = thread::spawn(move || {
            let _ = metrics_worker.listen();
        });
        metrics_sender
    }

    //generates aggregations (1-min-interval each) for a given amount of days, incrementing values by one
    fn generate_aggregation_for_days(
        n_days: u32,
        metrics_sender: &MetricsSender,
        key: &String,
        starting_datetime_y_m_d: (i32, u32, u32),
    ) {
        let mut value = 1;

        let (s_year, s_month, mut s_day) = starting_datetime_y_m_d;
        for _day in 0..n_days {
            for hour in 0..24 {
                for minute in 0..60 {
                    generate_and_send_aggregation(
                        metrics_sender,
                        key,
                        value,
                        (s_year, s_month, s_day),
                        (hour, minute, 0),
                    );
                    value += 1;
                }
            }
            s_day += 1;
        }
    }

    fn generate_and_send_aggregation(
        metrics_sender: &MetricsSender,
        key: &String,
        value: i32,
        datetime_y_m_d: (i32, u32, u32),
        datetime_h_m_s: (u32, u32, u32),
    ) {
        let (year, month, day) = datetime_y_m_d;
        let (hour, minute, second) = datetime_h_m_s;
        let mut agg: HashMap<String, i32> = HashMap::new();
        let naive_datetime = NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second);
        let datetime: DateTime<Local> = Local.from_local_datetime(&naive_datetime).unwrap();
        agg.insert(key.clone(), value);
        metrics_sender.update(agg, datetime);
    }

    fn json_response_from_points(points: Vec<(&str, i32)>) -> String {
        let mut json_response = r#"{"data":["#.to_string();
        for (moment, value) in points {
            let addition = format!(
                "{{\"moment\":\"{}\",\"value\":{}}},",
                moment,
                value.to_string()
            );
            json_response = json_response + &addition;
        }
        json_response.pop();
        json_response.push_str(r#"]}"#);
        json_response
    }

    #[test]
    fn invalid_key() {
        let key = "invalid_key".to_string();
        let metrics_sender = setup(1);

        let response = metrics_sender
            .get_metrics_response(key, TimeFrame::LastHours(1), GroupBy::Minutes(1))
            .unwrap();

        assert_eq!(response, METRIC_NOT_FOUND);
    }

    #[test]
    fn requested_1d_having_recorded_2d_while_store_limit_is_3d() {
        let (requested_days, recorded_days, store_limit_days) = (1, 2, 3);

        let key = TORRENTS_STAT.to_string();
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);
        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastDays(requested_days),
                GroupBy::Minutes(1),
            )
            .unwrap();

        assert_eq!(response.matches("moment").count(), 24 * 60);
    }

    #[test]
    fn requested_2d_having_recorded_1d_while_store_limit_is_1d() {
        let (requested_days, recorded_days, store_limit_days) = (2, 1, 1);

        let key = TORRENTS_STAT.to_string();
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);
        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastDays(requested_days),
                GroupBy::Minutes(1),
            )
            .unwrap();

        assert_eq!(response.matches("moment").count(), 24 * 60);
    }

    #[test]
    fn requested_2d_having_recorded_2d_while_store_limit_is_5d() {
        let (requested_days, recorded_days, store_limit_days) = (2, 2, 5);

        let key = TORRENTS_STAT.to_string();
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);
        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastDays(requested_days),
                GroupBy::Minutes(1),
            )
            .unwrap();

        assert_eq!(
            response.matches("moment").count(),
            24 * 60 * requested_days as usize
        );
    }

    #[test]
    fn timeframe_is_correct_by_hours() {
        let (requested_hours, recorded_days, store_limit_days) = (3, 1, 1);

        let key = TORRENTS_STAT.to_string();
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);

        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastHours(requested_hours),
                GroupBy::Minutes(1),
            )
            .unwrap();

        assert_eq!(
            response.matches("moment").count(),
            60 * requested_hours as usize
        );
    }

    #[test]
    fn groupby_minutes_gets_correct_amount_of_points() {
        let (requested_hours, recorded_days, store_limit_days) = (2, 1, 1);
        let groupby_minutes = 10;

        let key = TORRENTS_STAT.to_string();
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);
        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastHours(requested_hours),
                GroupBy::Minutes(groupby_minutes),
            )
            .unwrap();

        assert_eq!(
            response.matches("moment").count(),
            ((60 / groupby_minutes) * requested_hours) as usize
        );
    }

    #[test]
    fn groupby_hours_gets_correct_amount_of_points() {
        let (requested_hours, recorded_days, store_limit_days) = (10, 1, 1);
        let groupby_hours = 2;

        let key = TORRENTS_STAT.to_string();
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);
        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastHours(requested_hours),
                GroupBy::Hours(groupby_hours),
            )
            .unwrap();

        assert_eq!(
            response.matches("moment").count(),
            (requested_hours / groupby_hours) as usize
        );
    }

    #[test]
    fn groupby_method_for_active_peers() {
        let (requested_days, recorded_days, store_limit_days) = (1, 1, 1);
        let groupby_hours = 6;

        let key = "fake_info_hash.".to_string() + ACTIVE_PEERS_STAT;
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);
        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastDays(requested_days),
                GroupBy::Hours(groupby_hours),
            )
            .unwrap();

        let expected_points = vec![
            ("2022-01-01 03:00:00", 180),
            ("2022-01-01 09:00:00", 540),
            ("2022-01-01 15:00:00", 900),
            ("2022-01-01 21:00:00", 1260),
        ];

        let expected_response = json_response_from_points(expected_points);
        assert_eq!(response, expected_response);
    }

    #[test]
    fn groupby_method_for_completed_downloads_and_torrents() {
        let (requested_days, recorded_days, store_limit_days) = (1, 1, 1);
        let groupby_hours = 6;

        let key = "fake_info_hash.".to_string() + COMPLETED_DOWNLOADS_STAT;
        let metrics_sender = setup(store_limit_days);
        generate_aggregation_for_days(recorded_days, &metrics_sender, &key, STARTING_Y_M_D);
        let response = metrics_sender
            .get_metrics_response(
                key,
                TimeFrame::LastDays(requested_days),
                GroupBy::Hours(groupby_hours),
            )
            .unwrap();

        let expected_points = vec![
            ("2022-01-01 03:00:00", 360),
            ("2022-01-01 09:00:00", 720),
            ("2022-01-01 15:00:00", 1080),
            ("2022-01-01 21:00:00", 1440),
        ];

        let expected_response = json_response_from_points(expected_points);
        assert_eq!(response, expected_response);
    }
}
