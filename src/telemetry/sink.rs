use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct Sink {
    recorded_data: Arc<Mutex<HashMap<String, Option<Value>>>>,
}

impl Sink {
    pub fn new() -> Self {
        Self {
            recorded_data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn record<S: Into<String>>(&mut self, signal: S) {
        let mut lock = self.recorded_data.lock().expect("mutex to not be poisoned");

        lock.entry(signal.into()).or_insert(None);
    }

    pub fn record_data<S: Into<String>>(&mut self, signal: S, data: Value) {
        let mut lock = self.recorded_data.lock().expect("mutex to not be poisoned");

        lock.entry(signal.into()).or_insert(Some(data));
    }

    pub fn recorded<S: Into<String>>(&self, signal: S) -> bool {
        let lock = self.recorded_data.lock().expect("mutex to not be poisoned");

        lock.contains_key(&signal.into())
    }

    pub fn data_recorded<S: Into<String>>(&self, signal: S) -> Value {
        let lock = self.recorded_data.lock().expect("mutex to not be poisoned");

        lock.get(&signal.into())
            .and_then(|value| value.clone())
            .unwrap_or(Value::Null)
    }
}
