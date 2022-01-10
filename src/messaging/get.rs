use microserde::json::{Number, Value};

use std::collections::HashMap;

use crate::messaging::Message;

pub trait Get: GetTelemetry {
    fn get(&mut self, _category: &str) -> Vec<Message>;
}

pub trait GetTelemetry {
    fn get_count(&self) -> u64;
    fn record_get(&mut self);
    fn get_messages_count(&self) -> u64;
    fn record_got_messages(&mut self, messages: &[Message]);
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

    pub(crate) fn queue_messages(&mut self, messages: &[Message]) {
        self.messages.extend_from_slice(messages)
    }
}

impl Get for SubstituteGetter {
    fn get(&mut self, _category: &str) -> Vec<Message> {
        self.record_get();
        if self.messages.len() > 0 {
            let messages = std::mem::replace(&mut self.messages, Vec::new());
            self.record_got_messages(&messages);

            messages
        } else {
            vec![]
        }
    }
}

impl GetTelemetry for SubstituteGetter {
    fn get_count(&self) -> u64 {
        self.telemetry
            .get("get_count")
            .map(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn record_get(&mut self) {
        self.telemetry
            .entry("get_count".to_string())
            .and_modify(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count += 1;
                } else {
                    *value = Value::Number(Number::U64(1));
                }
            })
            .or_insert(Value::Number(Number::U64(1)));
    }

    fn get_messages_count(&self) -> u64 {
        self.telemetry
            .get("get_messages_count")
            .map(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn record_got_messages(&mut self, messages: &[Message]) {
        let fetched_count = messages.len() as u64;
        self.telemetry
            .entry("get_messages_count".to_string())
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
    use crate::controls::*;

    #[test]
    // #[ignore]
    fn should_respond_to_fetch_with_queued_messages() {
        let messages = messages::example();
        let mut get = SubstituteGetter::new();
        get.queue_messages(&messages);
        let returned_messages = get.get("my-category");
        assert_eq!(messages, returned_messages);
    }
}
