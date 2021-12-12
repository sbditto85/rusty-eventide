pub mod messaging;
pub mod settings;

use messaging::*;
use settings::*;

pub struct Consumer<R: Read> {
    category: String,
    reader: R,
}

impl Consumer<SubstituteReader> {
    pub fn new(category: &str) -> Consumer<SubstituteReader> {
        Consumer {
            category: category.to_string(),
            reader: SubstituteReader::new(),
        }
    }
}

impl Consumer<PostgresReader> {
    pub fn build(category: &str) -> Consumer<PostgresReader> {
        Consumer {
            category: category.to_string(),
            reader: PostgresReader,
        }
    }
}

impl<R: Read> Consumer<R> {
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
        let messages = self.reader.fetch_messages(&self.category);
    }

    fn reader(&self) -> &impl Read {
        &self.reader
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
    fn should_periodically_ask_for_messages() {
        let mut consumer = Consumer::new("mycategory");

        consumer.tick();

        let reader = consumer.reader();

        assert!(reader.fetch_count() > 0);
    }
}
