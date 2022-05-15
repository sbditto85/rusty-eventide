use super::{PositionStore, PositionStoreTelemetry};

pub fn stream_name(category: &str) -> String {
    "".to_string()
}

#[derive(Debug)]
pub struct PostgresPositionStore {
    category: String,
}

impl PostgresPositionStore {
    pub fn build(category: impl Into<String>) -> Self {
        Self {
            category: category.into(),
        }
    }
}

impl PositionStore for PostgresPositionStore {
    fn get(&mut self) -> u64 {
        1
    }
    fn put(&mut self, _position: u64) {}
}

impl PositionStoreTelemetry for PostgresPositionStore {
    fn record_get(&mut self) {}

    fn record_put(&mut self) {}
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::{controls, settings::Settings};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn should_store_position_in_the_category_with_a_position_type() {
        init();

        // Arrange
        let category = controls::messages::category::unique_category();
        let mut position_store = PostgresPositionStore::build(&category);
        let position = 1;
        let position_stream_name = stream_name(&category);

        // Act
        position_store.put(position);

        // Assert
        // TODO: Read from the `category:position` stream (no consumer identity provided) and expect a `Recorded` event with a `position` attribute
    }

    #[test]
    #[ignore]
    fn should_use_consumer_identity_as_stream_identity_if_provided() {}
}
