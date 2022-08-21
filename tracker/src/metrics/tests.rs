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
