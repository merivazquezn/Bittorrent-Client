use chrono::prelude::*;
pub trait ChunkAggregator {
    fn aggregate(&self, chunk: &[(i32, DateTime<Local>)]) -> i32;
}

pub enum AggregatingMethod {
    Average,
    Max,
}

fn aggregate_average(chunk: &[(i32, DateTime<Local>)]) -> i32 {
    let mut avg = 0;
    for (value, _timestamp) in chunk {
        avg += value
    }
    avg / (chunk.len()) as i32
}

fn aggregate_max(chunk: &[(i32, DateTime<Local>)]) -> i32 {
    let mut max_value = 0;
    for (value, _timestamp) in chunk {
        if *value > max_value {
            max_value = *value
        }
    }
    max_value
}

impl ChunkAggregator for AggregatingMethod {
    fn aggregate(&self, chunk: &[(i32, DateTime<Local>)]) -> i32 {
        match self {
            AggregatingMethod::Average => aggregate_average(chunk),
            AggregatingMethod::Max => aggregate_max(chunk),
        }
    }
}
