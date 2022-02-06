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
    fn duration(&mut self) -> Duration {
        self.duration
    }
}