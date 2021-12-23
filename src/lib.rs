pub mod controls;
pub mod messaging;
pub mod settings;

use messaging::*;
use settings::*;

pub struct Consumer<G: Get> {
    category: String,
    get: G,
}

impl Consumer<SubstituteGetter> {
    pub fn new(category: &str) -> Consumer<SubstituteGetter> {
        Consumer {
            category: category.to_string(),
            get: SubstituteGetter::new(),
        }
    }
}

impl Consumer<PostgresGetter> {
    pub fn build(category: &str) -> Consumer<PostgresGetter> {
        Consumer {
            category: category.to_string(),
            get: PostgresGetter,
        }
    }
}

impl<R: Get> Consumer<R> {
    pub fn add_handler<H: messaging::Handler>(self, _handler: H) -> Self {
        self
    }

    pub fn with_settings(self, _settings: Settings) -> Self {
        self
    }

    pub fn start(self) -> ConsumerHandler {
        ConsumerHandler::new()
    }

    fn tick(&mut self) {
        let messages = self.get.fetch_messages(&self.category);
    }

    fn get(&self) -> &impl Get {
        &self.get
    }

    fn get_mut(&mut self) -> &mut impl Get {
        &mut self.get
    }
}

pub struct ConsumerHandler {
    stopped: bool,
}

impl ConsumerHandler {
    pub fn new() -> Self {
        Self { stopped: false }
    }

    pub fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
    }

    pub fn started(&self) -> bool {
        !self.stopped
    }

    pub fn stopped(&self) -> bool {
        self.stopped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_ask_for_messages_every_tick() {
        let mut consumer = Consumer::new("mycategory");

        consumer.tick();

        let get = consumer.get();

        assert!(get.fetch_count() > 0);
    }

    #[test]
    fn should_return_queued_messages_on_tick() {
        let mut consumer = Consumer::new("mycategory");

        let get = consumer.get_mut();
        let messages = controls::messages::example();
        let messages_count = messages.len() as u64;
        get.queue_messages(messages);

        consumer.tick();

        let get = consumer.get();

        assert_eq!(messages_count, get.fetched_messages_count());
    }
}
