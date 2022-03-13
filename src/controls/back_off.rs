use std::time::Duration;

use crate::back_off::BackOff;

pub struct OnNoMessageDataCount {
    duration: Duration,
}

impl OnNoMessageDataCount {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl BackOff for OnNoMessageDataCount {
    fn duration(&mut self, iteration_message_count: u64) -> Duration {
        if iteration_message_count > 0 {
            Duration::ZERO
        } else {
            self.duration
        }
    }
}
