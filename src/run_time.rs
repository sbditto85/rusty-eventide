use std::time::Duration;

pub trait RunTime {
    fn sleep(&mut self, duration: Duration);

    fn set_run_limit(&mut self, total_run_time: u64);

    fn should_continue(&mut self) -> bool;
}

pub struct SubstituteRunTime {}

impl SubstituteRunTime {
    pub fn new() -> Self {
        Self {}
    }
}

impl RunTime for SubstituteRunTime {
    fn sleep(&mut self, duration: Duration) {}

    fn set_run_limit(&mut self, total_run_time: u64) {}

    fn should_continue(&mut self) -> bool {
        false
    }
}

pub struct SystemRunTime {}

impl SystemRunTime {
    pub fn build() -> Self {
        Self {}
    }
}

impl RunTime for SystemRunTime {
    fn sleep(&mut self, duration: Duration) {}

    fn set_run_limit(&mut self, total_run_time: u64) {}

    fn should_continue(&mut self) -> bool {
        false
    }
}
