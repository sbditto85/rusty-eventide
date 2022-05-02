use serde_json::Value;

use std::collections::HashMap;

use crate::consumer::DEFAULT_POSITION;

pub mod postgres;

pub trait PositionStore: PositionStoreTelemetry + std::fmt::Debug {
    fn get(&mut self) -> u64;
    fn put(&mut self, position: u64); //TODO: should have result?
}

pub trait PositionStoreTelemetry {
    fn record_get(&mut self);
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

    pub fn get_count(&self) -> u64 {
        self.telemetry
            .get(GET_COUNT_KEY)
            .map(|value| {
                if let Some(count) = value.as_u64() {
                    count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }

    pub fn put_count(&self) -> u64 {
        self.telemetry
            .get(PUT_COUNT_KEY)
            .map(|value| {
                if let Some(count) = value.as_u64() {
                    count
                } else {
                    0
                }
            })
            .unwrap_or(0)
    }
}

impl PositionStore for SubstitutePositionStore {
    fn get(&mut self) -> u64 {
        self.record_get();
        self.position.unwrap_or(DEFAULT_POSITION)
    }
    fn put(&mut self, _position: u64) {
        self.record_put();
    }
}

const GET_COUNT_KEY: &'static str = "get_count";
const PUT_COUNT_KEY: &'static str = "put_count";

impl PositionStoreTelemetry for SubstitutePositionStore {
    fn record_get(&mut self) {
        self.telemetry
            .entry(GET_COUNT_KEY.to_string())
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

    fn record_put(&mut self) {
        self.telemetry
            .entry(PUT_COUNT_KEY.to_string())
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
}
