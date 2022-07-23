use super::super::types::MetricsMessage;
use crate::metrics::params::*;
use crate::http::IHttpService;
use chrono::prelude::*;
use chrono::DurationRound;
use chrono::Duration;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvError};

pub struct MetricsWorker {
    pub receiver: Receiver<MetricsMessage>,
    pub record: HashMap<String, Vec<(i32, String)>>, 
    pub store_minutes: usize,
}

fn timestamp_to_string(timestamp: DateTime<Local>) -> String {
    timestamp.duration_round(Duration::minutes(1)).unwrap().format("%Y-%m-%d %H:%M:%S").to_string()
}

impl MetricsWorker {
    fn send_metric(
        &self,
        _https_service: Box<dyn IHttpService>,
        _key: String,
        _time_frame: TimeFrame,
        _groupby: GroupBy,
    ) {
        //send JSON through IHttpsService
    }

    fn update(&mut self, aggregation: HashMap<String, i32>, timestamp: DateTime<Local>) {
        let timestamp_string = timestamp_to_string(timestamp);
        for (key, value) in aggregation.iter(){
            if !self.record.contains_key(key){
                self.record.insert(key.clone(), vec![(value.clone(), timestamp_string.clone())]);
                continue;
            }

            let record_vector =  self.record.get_mut(key).unwrap();

            record_vector.push((value.clone(), timestamp_string.clone()));
            
            if record_vector.len() > self.store_minutes{
                record_vector.remove(0);
            }
        }
    }

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
}
