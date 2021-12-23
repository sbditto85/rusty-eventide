use microserde::json::{Number, Value};

use std::collections::HashMap;

use crate::messaging::Message;

pub trait Get: GetTelemetry {
    fn fetch_messages(&mut self, _category: &str) -> ();

    fn queue_messages(&mut self, _messages: &[Message]);
}

pub trait GetTelemetry {
    fn fetch_count(&self) -> u64;
    fn record_fetch(&mut self);
    fn fetched_messages_count(&self) -> u64;
    fn record_fetched_messages(&mut self, messages: &[Message]);
}

pub struct SubstituteGetter {
    messages: Vec<Message>,
    telemetry: HashMap<String, Value>,
}

impl SubstituteGetter {
    pub(crate) fn new() -> Self {
        Self {
            messages: vec![],
            telemetry: HashMap::new(),
        }
    }
}

impl Get for SubstituteGetter {
    fn fetch_messages(&mut self, _category: &str) -> () {
        self.record_fetch();
        if self.messages.len() > 0 {
            let messages = std::mem::replace(&mut self.messages, Vec::new());
            self.record_fetched_messages(&messages);
        }
    }

    fn queue_messages(&mut self, messages: &[Message]) {
        self.messages.extend_from_slice(messages)
    }
}

impl GetTelemetry for SubstituteGetter {
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

    fn record_fetched_messages(&mut self, messages: &[Message]) {
        let fetched_count = messages.len() as u64;
        self.telemetry
            .entry("fetched_messages_count".to_string())
            .and_modify(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count += fetched_count;
                } else {
                    *value = Value::Number(Number::U64(fetched_count));
                }
            })
            .or_insert(Value::Number(Number::U64(fetched_count)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn should_respond_to_fetch_with_queued_messages() {
        assert!(false);
    }
}
