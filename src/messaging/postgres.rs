// use std::collections::HashMap;
use std::error::Error as StdError;

use thiserror::Error;

use crate::{
    messaging::{
        get::{Get, GetError, GetTelemetry},
        MessageData,
    },
    session::Session,
};

#[derive(Error, Debug)]
pub enum CategoryError {
    #[error("Session Error: {0}")]
    SessionError(#[from] crate::session::SessionError),
}

#[derive(Debug)]
pub struct Category {
    category: String,
    session: Session, // telemetry: HashMap::new(),
}

impl Category {
    pub fn build(category: impl Into<String>) -> Result<Self, CategoryError> {
        Ok(Self {
            category: category.into(),
            session: Session::build()?,
        })
    }
}

//TODO: actually do this
impl Get for Category {
    fn get(&mut self, position: i64) -> Result<Vec<MessageData>, GetError> {
        // self.record_get();

        /*
        category varchar,
        "position" bigint DEFAULT 1,
        batch_size bigint DEFAULT 1000,
        correlation varchar DEFAULT NULL,
        consumer_group_member bigint DEFAULT NULL,
        consumer_group_size bigint DEFAULT NULL,
        condition varchar DEFAULT NULL
         */
        let batch_size: Option<i64> = None;
        let correlation: Option<String> = None;
        let consumer_group_member: Option<i64> = None;
        let consumer_group_size: Option<i64> = None;
        let condition: Option<String> = None;

        let rows = self.session
            .query("SELECT * FROM get_category_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar, $5::bigint, $6::bigint, $7::varchar);", 
            &[ &self.category, &position, &batch_size, &correlation, &consumer_group_member, &consumer_group_size, &condition ])
            .map_err(|error| {
                log::error!("THIS ERROR HAPPENED: {}", error);
                Box::new(error) as Box<dyn StdError + Send + Sync>
            })?;

        Ok(rows
            .into_iter()
            .map(|_row| {
                // TODO: test force getting message data
                // let global_position: i64 = row.get("global_position");
                MessageData { global_position: 0 }
            })
            .collect())
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

    fn record_got_messages(&mut self, _messages: &[MessageData]) {}
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::controls;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn should_get_no_messages_when_none_available() {
        init();

        // Arrange
        let category = controls::category::unique_category();
        let mut category_get = Category::build(category).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let no_messages = 0;
        assert_eq!(messages.len(), no_messages);
    }

    #[test]
    fn should_get_one_message_when_one_in_stream() {
        init();

        // Arrange
        let category = controls::messages::write_random_to_random_category();
        let mut category_get = Category::build(category).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let one_message = 1;
        assert_eq!(messages.len(), one_message);
    }

    // TODO: put in a writer test
    // #[test]
    // #[ignore]
    // fn should_properly_return_an_expected_version_error_when_expected_version_is_incorrect() {}

    #[test]
    fn should_get_multiple_messages_when_multiple_in_stream() {
        init();

        // Arrange
        let category = controls::messages::write_random_to_random_category();
        controls::messages::write_random_to_category(&category);
        let mut category_get = Category::build(category).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let multiple_messages = 2;
        assert_eq!(messages.len(), multiple_messages);
    }

    #[test]
    #[ignore]
    fn should_get_none_when_position_more_than_in_stream() {}

    #[test]
    #[ignore]
    fn should_get_half_when_position_half_way() {}

    #[test]
    #[ignore]
    fn should_limit_get_when_batch_size_less_then_in_stream() {}

    #[test]
    #[ignore]
    fn should_filter_by_correlation_when_applied() {}

    #[test]
    #[ignore]
    fn should_get_only_consumer_specific_messages_when_in_consumer_group() {}

    #[test]
    #[ignore]
    fn should_get_only_applicable_message_when_condition_supplied() {}
}
