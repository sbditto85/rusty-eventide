use crate::messaging::{
    get::{Get, GetTelemetry},
    Message,
};

pub struct Category;

//TODO: actually do this
impl Get for Category {
    fn get(&mut self, _position: i64) -> Vec<Message> {
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

    fn record_got_messages(&mut self, _messages: &[Message]) {}
}
