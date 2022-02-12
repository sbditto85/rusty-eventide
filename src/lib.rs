use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use back_off::{constant::ConstantBackOff, BackOff};
use messaging::{postgres::Category, *};
use settings::*;

pub mod back_off;
pub mod controls;
pub mod messaging;
pub mod settings;

pub struct Consumer<G: Get, B: BackOff> {
    #[allow(dead_code)]
    category: String,
    handlers: Vec<Box<dyn Handler + Send>>,
    active: bool,
    iterations: u64,
    get: G,
    back_off: B,
}

impl Consumer<SubstituteGetter, ConstantBackOff> {
    pub fn new(category: &str) -> Consumer<SubstituteGetter, ConstantBackOff> {
        Consumer {
            category: category.to_string(),
            handlers: Vec::new(),
            active: true,
            iterations: 0,
            get: SubstituteGetter::new(category),
            back_off: ConstantBackOff::new(),
        }
    }
}

impl Consumer<Category, ConstantBackOff> {
    pub fn build(category: &str) -> Consumer<Category, ConstantBackOff> {
        Consumer {
            category: category.to_string(),
            handlers: Vec::new(),
            active: true,
            iterations: 0,
            get: Category,
            back_off: ConstantBackOff::build(),
        }
    }
}

impl<G: Get + Send + 'static, B: BackOff + Send + 'static> Consumer<G, B> {
    pub fn add_handler<H: messaging::Handler + Send + 'static>(mut self, handler: H) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    pub fn with_settings(self, _settings: Settings) -> Self {
        self
    }

    pub fn with_back_off<B2: BackOff>(self, back_off: B2) -> Consumer<G, B2> {
        // Is there a better way to do this? where I only have to specify back_off?
        // can't use `..self` because B and B2 are different types :(
        Consumer {
            category: self.category,
            handlers: self.handlers,
            active: self.active,
            iterations: self.iterations,
            get: self.get,
            back_off,
        }
    }

    pub fn start(self) -> ConsumerHandler<G, B> {
        let arc = Arc::new(Mutex::new(self));
        let thread_arc = arc.clone();

        let handle = std::thread::spawn(move || {
            loop {
                let mut consumer = thread_arc.lock().expect("the mutex to not be poisoned");

                if !consumer.deref().active {
                    break;
                }

                let iteration_message_count = consumer.tick();

                let wait_time = consumer.back_off.duration(iteration_message_count);

                // Give the main thread a chance to lock the mutex
                drop(consumer);
                std::thread::sleep(wait_time);
            }
        });

        ConsumerHandler::new(arc.clone(), handle)
    }

    pub fn tick(&mut self) -> u64 {
        self.iterations += 1;
        let messages = self.get.get(0); //TODO: handle position
        let messages_length = messages.len();

        for message in messages {
            for handler in &mut self.handlers {
                handler.handle(message.clone());
            }
        }

        messages_length as u64
    }

    pub fn get(&self) -> &G {
        &self.get
    }

    pub fn get_mut(&mut self) -> &mut G {
        &mut self.get
    }
}

pub struct ConsumerHandler<G: Get, B: BackOff> {
    consumer: Arc<Mutex<Consumer<G, B>>>,
    handle: Option<JoinHandle<()>>,
}

impl<G: Get, B: BackOff> ConsumerHandler<G, B> {
    pub fn new(consumer: Arc<Mutex<Consumer<G, B>>>, handle: JoinHandle<()>) -> Self {
        Self {
            consumer,
            handle: Some(handle),
        }
    }

    pub fn iterations(&self) -> u64 {
        let consumer = self.consumer.lock().expect("mutex to not be poisoned");
        consumer.iterations
    }

    pub fn stop(&mut self) {
        let mut consumer = self.consumer.lock().expect("mutex to not be poisoned");
        consumer.active = false;
        drop(consumer);

        self.handle.take().map(|thread| thread.join());
    }

    pub fn started(&self) -> bool {
        let consumer = self.consumer.lock().expect("mutex to not be poisoned");
        consumer.active
    }

    pub fn stopped(&self) -> bool {
        let consumer = self.consumer.lock().expect("mutex to not be poisoned");
        !consumer.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /////////////////////
    // Get
    /////////////////////

    #[test]
    fn should_ask_for_messages_every_tick() {
        let mut consumer = Consumer::new("mycategory");

        consumer.tick();

        let get = consumer.get();

        assert!(get.get_count() > 0);
    }

    #[test]
    fn should_return_same_number_of_queued_messages_on_tick() {
        let mut consumer = Consumer::new("mycategory");

        let get = consumer.get_mut();
        let messages = controls::messages::example();
        let messages_count = messages.len() as u64;
        get.queue_messages(&messages);

        consumer.tick();

        let get = consumer.get();

        assert_eq!(messages_count, get.get_messages_count());
    }

    /////////////////////
    // Running
    /////////////////////

    // Is this a good test? idk, feels a little like imperative shell to me
    #[test]
    fn should_continue_tick_until_stopped() {
        let mut consumer = Consumer::new("mycategory").start();

        assert!(consumer.started());
        let beginning = consumer.iterations();

        std::thread::sleep(std::time::Duration::from_millis(50));

        consumer.stop();
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        assert!(ending > beginning);

        std::thread::sleep(std::time::Duration::from_millis(50));

        assert_eq!(ending, consumer.iterations());
    }

    #[test]
    #[ignore]
    fn should_be_able_to_wait_until_consumer_is_done() {}

    /////////////////////
    // Back off
    /////////////////////

    #[test]
    fn should_be_able_to_specify_a_back_off_strategy() {
        let duration = std::time::Duration::from_millis(60);

        let mut consumer = Consumer::new("mycategory")
            .with_back_off(crate::back_off::constant::ConstantBackOff::new_with_duration(duration))
            .start();

        assert!(consumer.started());
        let beginning = consumer.iterations();

        std::thread::sleep(std::time::Duration::from_millis(50));

        consumer.stop();
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        // Only enough time to get one iteration off due to back off being longer then test sleep
        let expected_ending = beginning + 1;
        assert_eq!(expected_ending, ending);
    }

    #[test]
    fn should_be_able_to_use_last_message_count_to_determine_back_off() {
        let duration = std::time::Duration::from_millis(60);

        let mut consumer = Consumer::new("mycategory")
            .with_back_off(crate::controls::back_off::OnNoMessageCount::new(duration));

        let get = consumer.get_mut();
        let messages = controls::messages::example();
        get.queue_messages(&messages);

        let mut consumer = consumer.start();

        assert!(consumer.started());
        let beginning = consumer.iterations();

        std::thread::sleep(std::time::Duration::from_millis(50));

        consumer.stop();
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        // Only enough time to do one iteration with a message then immediately try for another
        //  which will cause a longer pause then the sleep between begin and end because no messages
        let expected_ending = beginning + 2;
        assert_eq!(expected_ending, ending);
    }

    /////////////////////
    // Handler
    /////////////////////

    #[test]
    fn should_offer_messages_to_handler_on_tick() {
        let handler = controls::handler::TrackingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        let get = consumer.get_mut();
        let messages = controls::messages::example();
        let messages_count = messages.len() as u64;
        get.queue_messages(&messages);

        consumer.tick();

        assert_eq!(handler.message_count(), messages_count);
    }

    #[test]
    #[ignore]
    fn should_stop_processing_messages_when_handler_errors_on_tick() {}

    /////////////////////
    // Position
    /////////////////////
    #[test]
    #[ignore]
    fn should_store_position_periodically_to_optimize_resume() {}
}
