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
        self.run_limit = self.run_limit.map(|limit| limit.saturating_sub(duration));
    }

    fn set_run_limit(&mut self, total_run_time: Duration) {
        self.run_limit = Some(total_run_time);
    }

    fn should_continue(&mut self) -> bool {
        if let Some(run_limit) = &self.run_limit {
            *run_limit > Duration::ZERO
        } else {
            true
        }
    }
}

#[derive(Debug)]
pub struct SystemRunTime {
    run_limit: Option<Duration>,
}

impl SystemRunTime {
    pub fn build() -> Self {
        Self { run_limit: None }
    }
}

impl RunTime for SystemRunTime {
    fn sleep(&mut self, duration: Duration) {
        self.run_limit = self.run_limit.map(|limit| limit.saturating_sub(duration));
        std::thread::sleep(duration);
    }

    fn set_run_limit(&mut self, total_run_time: Duration) {
        self.run_limit = Some(total_run_time);
    }

    fn should_continue(&mut self) -> bool {
        if let Some(run_limit) = &self.run_limit {
            *run_limit > Duration::ZERO
        } else {
            true
        }
    }
}
