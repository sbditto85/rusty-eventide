use rusty_eventide::{/*consumer::Consumer, */ messaging::HandleError, *};

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
    fn handle(&mut self, message: messaging::MessageData) -> Result<(), HandleError> {
        println!("Got a message: {:?}", message);
        Ok(())
    }
}

fn main() {
    // let consumer_handle = Consumer::build("category")
    //     .add_handler(EventHandler::build())
    //     .start();

    // let result = consumer_handle.wait();

    // println!("Finished with: {:?}", result);
}
