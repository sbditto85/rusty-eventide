use crate::messaging::{
    get::{Get, GetTelemetry},
    MessageData,
};

#[derive(Debug)]
pub struct Category;

//TODO: actually do this
impl Get for Category {
    fn get(&mut self, _position: i64) -> Vec<MessageData> {
        vec![]
    }
}

impl GetTelemetry for Category {
    fn get_count(&self) -> u64 {
        0
    }
    fn record_get(&mut self) {}

    fn get_messages_count(&self) -> u64 {
        0
    }

    fn record_got_messages(&mut self, _messages: &[MessageData]) {}
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    #[test]
    fn should_run() {}
}
