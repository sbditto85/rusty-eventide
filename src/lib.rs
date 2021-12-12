use actix::prelude::*;

pub mod messaging;
pub mod settings;

use settings::*;

#[derive(Message)]
#[rtype(result = "()")]
struct Stop;

pub struct Consumer;

impl Actor for Consumer {
    type Context = Context<Self>;
}

impl Handler<Stop> for Consumer {
    type Result = ();

    fn handle(&mut self, _msg: Stop, ctx: &mut Context<Self>) -> Self::Result {
        ctx.terminate();
    }
}

impl Consumer {
    pub fn new(_category: &str) -> Self {
        Consumer
    }

    pub fn build(_category: &str) -> Self {
        Consumer
    }

    pub fn add_handler<H: messaging::Handler>(self, _handler: H) -> Self {
        self
    }

    pub fn with_settings(self, _settings: Settings) -> Self {
        self
    }

    pub fn start(self) -> ConsumerHandler {
        let addr = Actor::start(self);

        ConsumerHandler::new(addr)
    }
}

pub struct ConsumerHandler {
    address: Addr<Consumer>,
    stopped: bool,
}

impl ConsumerHandler {
    pub fn new(address: Addr<Consumer>) -> Self {
        Self {
            address,
            stopped: false,
        }
    }

    pub fn stop(&mut self) {
        if self.stopped {
            return;
        }
        self.address.do_send(Stop);
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

    #[actix::test]
    async fn should_periodically_ask_for_messages() {
        let mut consumer = Consumer::new("mycategory");

        consumer.tick();

        let reader = consumer.reader();

        assert!(reader.request_count() > 0);
    }
}
