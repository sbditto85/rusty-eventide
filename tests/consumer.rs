use consumer::{settings::Settings, *};

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        EventHandler
    }

    pub fn build() -> Self {
        EventHandler
    }
}

impl messaging::Handler for EventHandler {}

#[actix::test]
async fn should_start_a_consumer() {
    let mut consumer = Consumer::build("category")
        .add_handler(EventHandler::build())
        .start();

    assert!(consumer.started());
    consumer.stop();
    assert!(consumer.stopped());
}

#[actix::test]
async fn should_start_a_consumer_with_settings() {
    let mut consumer = Consumer::build("category:command")
        .with_settings(Settings::new())
        .add_handler(EventHandler::build())
        .start();

    assert!(consumer.started());
    consumer.stop();
    assert!(consumer.stopped());
}
