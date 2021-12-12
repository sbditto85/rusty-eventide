use microserde::json::{Number, Value};

use std::collections::HashMap;

pub trait Read: ReadTelemetry {
    fn fetch_messages(&mut self, category: &str) -> () {
        self.record_fetch();
    }
}

pub trait ReadTelemetry {
    fn fetch_count(&self) -> u64;
    fn record_fetch(&mut self);
    // fn fetched_messages_count(&self) -> u64;
}

pub struct SubstituteReader {
    telemetry: HashMap<String, Value>,
}

impl SubstituteReader {
    pub(crate) fn new() -> Self {
        Self {
            telemetry: HashMap::new(),
        }
    }
}

impl Read for &SubstituteReader {}

impl ReadTelemetry for &SubstituteReader {
    fn fetch_count(&self) -> u64 {
        (*self).fetch_count()
    }

    fn record_fetch(&mut self) {
        (*self).record_fetch();
    }
}

impl Read for SubstituteReader {}

impl ReadTelemetry for SubstituteReader {
    fn fetch_count(&self) -> u64 {
        self.telemetry
            .get("fetch_count")
            .map(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn record_fetch(&mut self) {
        self.telemetry
            .entry("fetch_count".to_string())
            .and_modify(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count += 1;
                } else {
                    *value = Value::Number(Number::U64(1));
                }
            })
            .or_insert(Value::Number(Number::U64(1)));
    }
}

pub struct PostgresReader;

impl Read for &PostgresReader {}

impl ReadTelemetry for &PostgresReader {
    fn fetch_count(&self) -> u64 {
        (*self).fetch_count()
    }
    fn record_fetch(&mut self) {
        (*self).record_fetch();
    }
}

impl Read for PostgresReader {}

impl ReadTelemetry for PostgresReader {
    fn fetch_count(&self) -> u64 {
        0
    }
    fn record_fetch(&mut self) {}
}
