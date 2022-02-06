use std::time::Duration;

use crate::back_off::BackOff;

pub struct SpecificBackOff {
    duration: Duration,
}

impl SpecificBackOff {
    pub fn new(duration: Duration) -> Self {
        Self { duration }
    }
}

impl BackOff for SpecificBackOff {
    fn duration(&mut self, _iteration_message_count: u64) -> Duration {
        self.duration
    }
}

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
            Duration::from_millis(0)
        } else {
            self.duration
        }
    }
}
