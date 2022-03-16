use crate::messaging::{
    get::{Get, GetTelemetry},
    MessageData,
};

#[derive(Debug)]
pub struct Category;

impl Category {
    pub fn build(_category: impl Into<String>) -> Self {
        Self
    }
}

//TODO: actually do this
impl Get for Category {
    fn get(&mut self, _position: i64) -> Vec<MessageData> {
        vec![]
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

    #[test]
    fn should_get_no_messages_when_none_available() {
        // Arrange
        let mut category_get = Category::build("mycategory");

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position);

        // Assert
        let no_messages = 0;
        assert_eq!(messages.len(), no_messages);
    }

    #[test]
    #[ignore]
    fn should_get_one_message_when_one_available() {
        // Arrange
        let category = "mycategory";
        controls::messages::write_random(category);
        let mut category_get = Category::build(category);

        // Act
        let beginning_position = 0;
        let messages = category_get.get(beginning_position);

        // Assert
        let one_message = 1;
        assert_eq!(messages.len(), one_message);
    }

    #[test]
    #[ignore]
    fn should_get_multiple_messages_when_multiple_available() {}

    #[test]
    #[ignore]
    fn should_get_none_when_position_more_than_available() {}

    #[test]
    #[ignore]
    fn should_get_half_when_position_half_way() {}

    #[test]
    #[ignore]
    fn should_limit_get_when_batch_size_less_then_available() {}

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
