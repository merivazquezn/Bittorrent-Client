use super::super::types::TimerMessage;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct TimerSender {
    pub sender: Sender<TimerMessage>,
}

impl TimerSender {
    pub fn stop(&self) {
        let _ = self.sender.send(TimerMessage::Stop);
    }
}
