use microserde::json::{Number, Value};

use std::collections::HashMap;

pub mod postgres;

pub trait PositionStore: PositionStoreTelemetry + std::fmt::Debug {
    fn get(&mut self) -> u64;
    fn put(&mut self, position: u64); //TODO: should have result?
}

pub trait PositionStoreTelemetry {
    fn get_count(&self) -> u64;
    fn record_get(&mut self);
    fn put_count(&self) -> u64;
    fn record_put(&mut self);
}

#[derive(Debug)]
pub struct SubstitutePositionStore {
    position: Option<u64>,
    telemetry: HashMap<String, Value>,
}

impl SubstitutePositionStore {
    pub fn new() -> Self {
        Self {
            position: None,
            telemetry: HashMap::new(),
        }
    }

    pub fn set_position(&mut self, position: u64) {
        self.position = Some(position);
    }
}

impl PositionStore for SubstitutePositionStore {
    fn get(&mut self) -> u64 {
        self.record_get();
        self.position.unwrap_or(0)
    }
    fn put(&mut self, _position: u64) {
        self.record_put();
    }
}

const GET_COUNT_KEY: &'static str = "get_count";
const PUT_COUNT_KEY: &'static str = "put_count";

impl PositionStoreTelemetry for SubstitutePositionStore {
    fn get_count(&self) -> u64 {
        self.telemetry
            .get(GET_COUNT_KEY)
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
            .entry(GET_COUNT_KEY.to_string())
            .and_modify(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count += 1;
                } else {
                    *value = Value::Number(Number::U64(1));
                }
            })
            .or_insert(Value::Number(Number::U64(1)));
    }

    fn put_count(&self) -> u64 {
        self.telemetry
            .get(PUT_COUNT_KEY)
            .map(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    fn record_put(&mut self) {
        self.telemetry
            .entry(PUT_COUNT_KEY.to_string())
            .and_modify(|value| {
                if let Value::Number(Number::U64(count)) = value {
                    *count += 1;
                } else {
                    *value = Value::Number(Number::U64(1));
                }
            })
            .or_insert(Value::Number(Number::U64(1)));
    }
}
