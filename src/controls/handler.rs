use std::sync::{Arc, Mutex};

use crate::messaging::{Handler, Message};

#[derive(Debug, Clone)]
pub struct TrackingHandler {
    count: Arc<Mutex<u64>>,
}

impl Handler for TrackingHandler {
    fn handle(&mut self, _message: Message) {
        let mut count = self.count.lock().expect("mutex to not be poisoned");
        *count += 1;
    }
}

impl TrackingHandler {
    pub fn build() -> Self {
        Self {
            count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn message_count(&self) -> u64 {
        let count = self.count.lock().expect("mutex to not be poisoned");

        *count
    }
}
