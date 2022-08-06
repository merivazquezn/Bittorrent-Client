use crate::aggregator::timer::errors::TimerError;
#[derive(Debug)]
pub enum AggregatorError {
    ErrorReadingMessage(String),
    TimerError(TimerError),
}

impl From<std::sync::mpsc::RecvError> for AggregatorError {
    fn from(err: std::sync::mpsc::RecvError) -> Self {
        AggregatorError::ErrorReadingMessage(err.to_string())
    }
}

impl From<TimerError> for AggregatorError {
    fn from(err: TimerError) -> Self {
        AggregatorError::TimerError(err)
    }
}
