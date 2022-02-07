use std::time::Duration;

use crate::back_off::BackOff;

pub struct ConstantBackOff {
    duration: Duration,
}

impl ConstantBackOff {
    pub fn new() -> Self {
        Self {
            duration: Duration::from_millis(10),
        }
    }

    pub fn new_with_duration(duration: Duration) -> Self {
        Self {
            duration
        }
    }

    pub fn build() -> Self {
        Self {
            duration: Duration::from_millis(100),
        }
    }
}

impl BackOff for ConstantBackOff {
    fn duration(&mut self, _iteration_message_count: u64) -> Duration {
        self.duration
    }
}
