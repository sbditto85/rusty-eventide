use super::{PositionStore, PositionStoreTelemetry};

#[derive(Debug)]
pub struct PostgresPositionStore;

impl PostgresPositionStore {
    pub fn build() -> Self {
        Self
    }
}

impl PositionStore for PostgresPositionStore {
    fn put(&mut self, _position: u64) {}
}

impl PositionStoreTelemetry for PostgresPositionStore {
    fn put_count(&self) -> u64 {
        0
    }

    fn record_put(&mut self) {}
}
