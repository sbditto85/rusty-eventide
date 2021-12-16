use microserde::json::{Number, Value};

use std::collections::HashMap;

use crate::messaging::Message;

pub trait Read: ReadTelemetry {
    fn fetch_messages(&mut self, _category: &str) -> ();

    fn queue_messages(&mut self, _messages: Vec<Message>);
}

pub trait ReadTelemetry {
    fn fetch_count(&self) -> u64;
    fn record_fetch(&mut self);
    fn fetched_messages_count(&self) -> u64;
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

impl Read for SubstituteReader {
    fn fetch_messages(&mut self, _category: &str) -> () {
        self.record_fetch();
    }

    fn queue_messages(&mut self, _messages: Vec<Message>) {}
}

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

    fn fetched_messages_count(&self) -> u64 {
        self.telemetry
            .get("fetched_messages_count")
            .map(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }
}

pub struct PostgresReader;

//TODO: actually do this
impl Read for PostgresReader {
    fn fetch_messages(&mut self, _category: &str) -> () {}

    fn queue_messages(&mut self, _messages: Vec<Message>) {}
}

impl ReadTelemetry for PostgresReader {
    fn fetch_count(&self) -> u64 {
        0
    }
    fn record_fetch(&mut self) {}

    fn fetched_messages_count(&self) -> u64 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_respond_to_fetch_with_queued_messages() {
        assert!(false);
    }
}
