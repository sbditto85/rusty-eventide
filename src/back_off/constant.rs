use std::time::Duration;

use crate::back_off::BackOff;

#[derive(Debug)]
pub struct ConstantBackOff {
    duration: Duration,
}

impl ConstantBackOff {
    pub fn new() -> Self {
        Self {
            // Chosen to allow test to run quickly but still have some back off
            duration: Duration::from_millis(1),
        }
    }

    pub fn new_with_duration(duration: Duration) -> Self {
        Self { duration }
    }

    pub fn build() -> Self {
        Self {
            duration: Duration::from_millis(100), //TODO: Arbitrarily chosen, verify this makes sense or change
        }
    }
}

impl BackOff for ConstantBackOff {
    fn duration(&mut self, _iteration_message_count: u64) -> Duration {
        self.duration
    }
}
