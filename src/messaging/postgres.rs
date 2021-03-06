// use std::collections::HashMap;
use std::error::Error as StdError;

use thiserror::Error;

use crate::{
    messaging::{
        get::{Get, GetError, GetTelemetry},
        MessageData,
    },
    session::Session,
    settings::Settings,
};

#[derive(Error, Debug)]
pub enum CategoryError {
    #[error("Session Error: {0}")]
    SessionError(#[from] crate::session::SessionError),
}

#[derive(Debug)]
pub struct Category {
    category: String,
    settings: Settings,
    session: Session, // telemetry: HashMap::new(),
}

impl Category {
    pub fn build(category: impl Into<String>) -> Result<Self, CategoryError> {
        Ok(Self {
            category: category.into(),
            settings: Settings::build(),
            session: Session::build()?,
        })
    }

    pub fn build_params(
        category: impl Into<String>,
        settings: Settings,
        session: Session,
    ) -> Result<Self, CategoryError> {
        Ok(Self {
            category: category.into(),
            settings,
            session,
        })
    }
}

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
        let batch_size: Option<i64> = self.settings.batch_size.map(|bs| bs as i64);
        let correlation: &Option<String> = &self.settings.correlation;
        let consumer_group_member: Option<i64> =
            self.settings.consumer_group_member.map(|cgm| cgm as i64);
        let consumer_group_size: Option<i64> =
            self.settings.consumer_group_size.map(|cgs| cgs as i64);
        let condition: &Option<String> = &self.settings.condition;

        let rows = self.session
            .query("SELECT * FROM get_category_messages($1::varchar, $2::bigint, $3::bigint, $4::varchar, $5::bigint, $6::bigint, $7::varchar);", 
            &[ &self.category, &position, &batch_size, &correlation, &consumer_group_member, &consumer_group_size, &condition ])
            .map_err(|error| {
                log::error!("THIS ERROR HAPPENED: {}", error);
                Box::new(error) as Box<dyn StdError + Send + Sync>
            })?;

        log::trace!("Rows Returned: {:?}", rows);

        Ok(rows
            .into_iter()
            .map(|row| {
                let global_position: i64 = row.get("global_position");
                MessageData {
                    global_position: global_position as u64,
                }
            })
            .collect())
    }
}

impl GetTelemetry for Category {
    fn record_get(&mut self) {}

    fn record_got_messages(&mut self, _messages: &[MessageData]) {}
}

#[cfg(all(test, feature = "integration_tests"))]
mod integration_tests {
    use std::collections::HashMap;

    use super::*;
    use crate::{controls, settings::Settings};

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
        let category = controls::messages::postgres::write_random_message_to_random_category();
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
        let category = controls::messages::postgres::write_random_message_to_random_category();
        controls::messages::postgres::write_random_message_to_category(&category);
        let mut category_get = Category::build(category).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let multiple_messages = 2;
        assert_eq!(messages.len(), multiple_messages);
    }

    #[test]
    fn should_limit_get_when_batch_size_less_then_in_stream() {
        init();
        let batch_size = 2;

        // Arrange
        let category = controls::messages::postgres::write_random_message_to_random_category();
        controls::messages::postgres::write_bulk_random_messages_to_category(
            &category,
            batch_size * 2,
        );

        let category_count = controls::messages::postgres::category_count(&category);
        assert!(
            batch_size < category_count,
            "batch_size ({}) must be less then category_count ({}) for the test to work",
            batch_size,
            category_count
        );

        let mut settings = Settings::new();
        settings.batch_size = Some(batch_size);

        let session = Session::build().expect("session to be built");

        let mut category_get =
            Category::build_params(category, settings, session).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        assert_eq!(messages.len(), batch_size as usize);
    }

    #[test]
    fn should_filter_by_correlation_stream_name_when_applied() {
        init();

        // Arrange
        let correlation = "my_stream_category";

        let category = controls::messages::postgres::write_random_message_to_random_category();
        controls::messages::postgres::write_random_message_with_correlation_to_category(
            &category,
            &correlation,
        );

        let category_count = controls::messages::postgres::category_count(&category);
        let expected_total_count = 2;
        assert_eq!(category_count, expected_total_count);

        let mut settings = Settings::new();
        settings.correlation = Some(correlation.to_string());

        let session = Session::build().expect("session to be built");

        let mut category_get =
            Category::build_params(category, settings, session).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let correlation_count = 1;
        assert_eq!(messages.len(), correlation_count);
    }

    #[test]
    fn should_get_only_consumer_specific_messages_when_in_consumer_group() {
        init();

        // Arrange
        let consumer_group_member = 0;
        let consumer_group_size = 2;

        let category =
            controls::messages::postgres::write_one_random_message_for_consumer_and_one_not_to_random_category(
                consumer_group_member,
                consumer_group_size,
            );

        let category_count = controls::messages::postgres::category_count(&category);
        let expected_more_than = 1;
        assert!(category_count > expected_more_than);

        let mut settings = Settings::new();
        settings.consumer_group_member = Some(consumer_group_member);
        settings.consumer_group_size = Some(consumer_group_size);

        let session = Session::build().expect("session to be built");

        let mut category_get =
            Category::build_params(category, settings, session).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        //Assert
        let consumer_message_count = 1;
        assert_eq!(messages.len(), consumer_message_count);
    }

    #[test]
    fn should_get_only_applicable_message_when_condition_supplied() {
        init();

        // Arrange
        let mut data = HashMap::new();
        data.insert("test", "true");
        let condition = "data->>'test' = 'true'".to_string();

        let category =
            controls::messages::postgres::write_one_random_message_with_data_to_category(data);

        controls::messages::postgres::write_random_message_to_category(&category);

        let category_count = controls::messages::postgres::category_count(&category);
        let expected_more_than = 1;
        assert!(category_count > expected_more_than);

        let mut settings = Settings::new();
        settings.condition = Some(condition);

        let mut session = Session::build().expect("to build a session");

        controls::messages::postgres::enable_condition_for_session(&mut session);

        let mut category_get =
            Category::build_params(category, settings, session).expect("category to build");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let consumer_message_count = 1;
        assert_eq!(messages.len(), consumer_message_count);
    }

    #[test]
    fn should_get_none_when_position_more_than_in_stream() {
        init();

        // Arrange
        let category = controls::messages::postgres::write_random_message_to_random_category();

        let category_count = controls::messages::postgres::category_count(&category);
        let expected_more_than = 0;
        assert!(category_count > expected_more_than);

        let current_max_global_position =
            controls::messages::postgres::current_max_global_position();

        let mut category_get = Category::build(category).expect("category to build");

        // Act
        let beginning_position = current_max_global_position + 1; // Should be at least one more then the message written above (could be more)
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let no_messages = 0;
        assert_eq!(messages.len(), no_messages);
    }

    #[test]
    fn should_get_half_when_position_half_way() {
        init();

        // Arrange
        let category = controls::messages::postgres::write_random_message_to_random_category();
        controls::messages::postgres::write_bulk_random_messages_to_category(&category, 2);

        let middle_global_position = controls::messages::postgres::current_max_global_position();

        controls::messages::postgres::write_bulk_random_messages_to_category(&category, 2);

        let category_count = controls::messages::postgres::category_count(&category);
        let expected_count = 5;
        assert!(category_count == expected_count);

        let mut category_get = Category::build(category).expect("category to build");

        // Act
        let beginning_position = middle_global_position + 1; // Get everything after the middle event
        let messages = category_get.get(beginning_position).expect("get to work");

        // Assert
        let two_messages = 2;
        assert_eq!(messages.len(), two_messages);
    }

    #[test]
    fn should_get_messages_with_global_position_set_correctly() {
        init();

        // Arrange
        let starting_global_position = controls::messages::postgres::current_max_global_position();

        let category = controls::messages::postgres::write_random_message_to_random_category();
        let category_count = controls::messages::postgres::category_count(&category);
        let expected_count = 1;
        assert!(category_count == expected_count);

        let mut category_get = Category::build(category).expect("category to build");

        // Act
        let messages = category_get
            .get(starting_global_position)
            .expect("get to work");

        // Assert
        for message in messages.into_iter() {
            assert!(message.global_position as i64 > starting_global_position);
        }
    }
}
