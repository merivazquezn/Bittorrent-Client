#[derive(Debug)]
pub enum TimerError {
    AggregatorDisconnected(String),
}

impl From<std::sync::mpsc::RecvTimeoutError> for TimerError {
    fn from(error: std::sync::mpsc::RecvTimeoutError) -> Self {
        TimerError::AggregatorDisconnected(error.to_string())
    }
}
