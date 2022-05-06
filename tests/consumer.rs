#[cfg(feature = "integration_tests")]
use rusty_eventide::settings::Settings;
use rusty_eventide::{messaging::HandleError, *};

#[derive(Debug)]
pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        EventHandler
    }

    pub fn build() -> Self {
        EventHandler
    }
}

impl messaging::Handler for EventHandler {
    fn handle(&mut self, _message: messaging::MessageData) -> Result<(), HandleError> {
        Ok(())
    }
}

// #[cfg(all(test, feature = "integration_tests"))]
// mod test {

//     use super::*;
//     use rusty_eventide::consumer::Consumer;

//     #[test]
//     fn should_start_a_consumer() {
//         let mut consumer = Consumer::build("category")
//             .add_handler(EventHandler::build())
//             .start();

//         assert!(consumer.started());
//         consumer.stop();
//         assert!(consumer.stopped());
//     }

//     #[test]
//     fn should_start_a_consumer_with_settings() {
//         let mut consumer = Consumer::build("category:command")
//             .with_settings(Settings::new())
//             .add_handler(EventHandler::build())
//             .start();

//         assert!(consumer.started());
//         consumer.stop();
//         assert!(consumer.stopped());
//     }
// }
