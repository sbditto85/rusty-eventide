use serde_json::Value;
use thiserror::Error;

use std::collections::HashMap;
use std::error::Error as StdError;

use crate::messaging::MessageData;

pub trait Get: GetTelemetry {
    fn get(&mut self, position: i64) -> Result<Vec<MessageData>, GetError>;
}

#[derive(Error, Debug)]
pub enum GetError {
    #[error("An error getting data occurred: {0}")]
    DataError(#[from] Box<dyn StdError + Send + Sync>),
}

pub trait GetTelemetry {
    fn get_count(&self) -> u64;
    fn record_get(&mut self);
    fn get_messages_count(&self) -> u64;
    fn record_got_messages(&mut self, messages: &[MessageData]);
}

#[derive(Debug)]
pub struct SubstituteGetter {
    #[allow(dead_code)]
    category: String,
    messages: Vec<MessageData>,
    telemetry: HashMap<String, Value>,
}

impl SubstituteGetter {
    pub fn new(category: &str) -> Self {
        Self {
            category: category.to_string(),
            messages: vec![],
            telemetry: HashMap::new(),
        }
    }

    pub fn queue_messages(&mut self, messages: &[MessageData]) {
        self.messages.extend_from_slice(messages)
    }
}

impl Get for SubstituteGetter {
    fn get(&mut self, position: i64) -> Result<Vec<MessageData>, GetError> {
        self.record_get();
        if self.messages.len() > 0 {
            let messages = std::mem::replace(&mut self.messages, Vec::new());
            let limited_messages = &messages[position as usize..];
            self.record_got_messages(&limited_messages);

            Ok(limited_messages.to_vec())
        } else {
            Ok(vec![])
        }
    }
}

impl GetTelemetry for SubstituteGetter {
    fn get_count(&self) -> u64 {
        self.telemetry
            .get("get_count")
            .map(|value| {
                if let Some(count) = value.as_u64() {
                    count
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
                if let Some(mut count) = value.as_u64() {
                    count += 1;
                    *value = count.into();
                } else {
                    *value = 1u64.into();
                }
            })
            .or_insert(1u64.into());
    }

    fn get_messages_count(&self) -> u64 {
        self.telemetry
            .get("get_messages_count")
            .map(|value| {
                if let Some(count) = value.as_u64() {
                    count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn record_got_messages(&mut self, messages: &[MessageData]) {
        let fetched_count = messages.len() as u64;
        self.telemetry
            .entry("get_messages_count".to_string())
            .and_modify(|value| {
                if let Some(mut count) = value.as_u64() {
                    count += fetched_count;
                    *value = count.into();
                } else {
                    *value = fetched_count.into();
                }
            })
            .or_insert(fetched_count.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::controls::*;

    #[test]
    fn should_respond_to_fetch_with_queued_messages() {
        let messages = messages::example();
        let mut get = SubstituteGetter::new("my_category");
        get.queue_messages(&messages);
        let returned_messages = get.get(0).expect("get to work");
        assert_eq!(messages, returned_messages);
    }

    #[test]
    fn should_respond_with_no_messages_when_none_queued() {
        let mut get = SubstituteGetter::new("my_category");
        let returned_messages = get.get(0).expect("get to work");
        assert!(returned_messages.len() == 0);
    }

    #[test]
    fn should_respond_to_fetch_with_queued_messages_respecting_position_as_index() {
        let messages = messages::example();
        let mut get = SubstituteGetter::new("my_category");
        get.queue_messages(&messages);
        let returned_messages = get.get(1).expect("get to work");
        assert_eq!(messages[1..], returned_messages);
    }

    #[test]
    fn should_record_a_count_for_each_get() {
        let mut get = SubstituteGetter::new("my_category");

        assert_eq!(get.get_count(), 0);
        get.get(0).expect("get to work");
        assert_eq!(get.get_count(), 1);
        get.get(0).expect("get to work");
        assert_eq!(get.get_count(), 2);
    }

    #[test]
    fn should_record_message_count_for_each_get() {
        let messages = messages::example();
        let messages_len = messages.len() as u64;
        let mut get = SubstituteGetter::new("my_category");

        assert_eq!(get.get_messages_count(), 0);
        get.queue_messages(&messages);
        let _ = get.get(0);
        assert_eq!(get.get_messages_count(), messages_len);

        get.queue_messages(&messages);
        let _ = get.get(0);
        assert_eq!(get.get_messages_count(), messages_len * 2);

        get.queue_messages(&messages);
        let _ = get.get(0);
        assert_eq!(get.get_messages_count(), messages_len * 3);
    }
}
