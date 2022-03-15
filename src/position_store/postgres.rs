use super::{PositionStore, PositionStoreTelemetry};

#[derive(Debug)]
pub struct PostgresPositionStore;

impl PostgresPositionStore {
    pub fn build() -> Self {
        Self
    }
}

impl PositionStore for PostgresPositionStore {
    fn get(&mut self) -> u64 {
        0
    }
    fn put(&mut self, _position: u64) {}
}

impl PositionStoreTelemetry for PostgresPositionStore {
    fn get_count(&self) -> u64 {
        0
    }

    fn record_get(&mut self) {}

    fn put_count(&self) -> u64 {
        0
    }

    fn record_put(&mut self) {}
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    #[test]
    fn should_run() {}
}
