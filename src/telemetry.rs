pub mod sink;

use sink::Sink;

pub struct Telemetry {
    sinks: Vec<Sink>,
}

impl Telemetry {
    pub fn new() -> Self {
        Self { sinks: Vec::new() }
    }

    pub fn register(&mut self, sink: Sink) {
        self.sinks.push(sink);
    }

    pub fn record<S: Into<String>>(&mut self, signal: S) {
        let signal_string = signal.into();
        for s in self.sinks.iter_mut() {
            s.record(signal_string.clone());
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    use crate::controls;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn should_send_recording_to_sink() {
        init();

        let mut telemetry = Telemetry::new();
        let mut sink = sink::Sink::new();
        telemetry.register(sink.clone());

        let signal = controls::telemetry::signal();

        telemetry.record(signal);

        assert!(sink.recorded(signal))
    }
}
