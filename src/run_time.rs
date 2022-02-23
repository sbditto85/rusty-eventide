use std::time::Duration;

pub trait RunTime {
    fn sleep(&mut self, duration: Duration);

    fn set_run_limit(&mut self, total_run_time: Duration);

    fn should_continue(&mut self) -> bool;
}

pub struct SubstituteRunTime {
    run_limit: Option<Duration>,
}

impl SubstituteRunTime {
    pub fn new() -> Self {
        Self { run_limit: None }
    }
}

impl RunTime for SubstituteRunTime {
    fn sleep(&mut self, duration: Duration) {
        self.run_limit = self.run_limit.map(|limit| limit - duration);
        println!("RUN_LIMIT: {:?}", self.run_limit);
    }

    fn set_run_limit(&mut self, total_run_time: Duration) {
        self.run_limit = Some(total_run_time);
    }

    fn should_continue(&mut self) -> bool {
        if let Some(run_limit) = &self.run_limit {
            *run_limit >= Duration::from_millis(0)
        } else {
            true
        }
    }
}

pub struct SystemRunTime {}

impl SystemRunTime {
    pub fn build() -> Self {
        Self {}
    }
}

impl RunTime for SystemRunTime {
    fn sleep(&mut self, _duration: Duration) {}

    fn set_run_limit(&mut self, _total_run_time: Duration) {}

    fn should_continue(&mut self) -> bool {
        false
    }
}
