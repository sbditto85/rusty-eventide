use std::time::Duration;

pub mod constant;

pub trait BackOff {
    fn duration(&mut self) -> Duration;
}
