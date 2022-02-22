use std::error::Error as StdError;
use std::sync::{Arc, Mutex};

use thiserror::Error;

use crate::messaging::{HandleError, Handler, Message};

#[derive(Debug, Clone)]
pub struct TrackingHandler {
    count: Arc<Mutex<u64>>,
}

impl Handler for TrackingHandler {
    fn handle(&mut self, _message: Message) -> Result<(), HandleError> {
        let mut count = self.count.lock().expect("mutex to not be poisoned");
        *count += 1;

        Ok(())
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

#[derive(Debug, Clone)]
pub struct FailingHandler {
    count: Arc<Mutex<u64>>,
}

#[derive(Error, Debug)]
pub enum FailingHandlerError {
    #[error("Forced an error to happen")]
    Forced,
}

impl Handler for FailingHandler {
    fn handle(&mut self, _message: Message) -> Result<(), HandleError> {
        let mut count = self.count.lock().expect("mutex to not be poisoned");
        *count += 1;

        Err((Box::new(FailingHandlerError::Forced) as Box<dyn StdError + Send>).into())
    }
}

impl FailingHandler {
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
