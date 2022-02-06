use std::time::Duration;

pub mod constant;

pub trait BackOff {
    fn duration(&mut self, iteration_message_count: u64) -> Duration;
}
