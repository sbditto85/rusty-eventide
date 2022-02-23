use std::time::Duration;

use crate::back_off::BackOff;

pub struct OnNoMessageCount {
    duration: Duration,
}

impl OnNoMessageCount {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl BackOff for OnNoMessageCount {
    fn duration(&mut self, iteration_message_count: u64) -> Duration {
        if iteration_message_count > 0 {
            Duration::ZERO
        } else {
            self.duration
        }
    }
}
