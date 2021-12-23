use crate::messaging::{
    get::{Get, GetTelemetry},
    Message,
};

pub struct Category;

//TODO: actually do this
impl Get for Category {
    fn fetch_messages(&mut self, _category: &str) -> () {}

    fn queue_messages(&mut self, _messages: &[Message]) {}
}

impl GetTelemetry for Category {
    fn fetch_count(&self) -> u64 {
        0
    }
    fn record_fetch(&mut self) {}

    fn fetched_messages_count(&self) -> u64 {
        0
    }

    fn record_fetched_messages(&mut self, _messages: &[Message]) {}
}
