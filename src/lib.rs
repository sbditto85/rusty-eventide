use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use messaging::{postgres::Category, *};
use settings::*;

pub mod controls;
pub mod messaging;
pub mod settings;

pub struct Consumer<G: Get> {
    #[allow(dead_code)]
    category: String,
    active: bool,
    iterations: u64,
    get: G,
}

impl Consumer<SubstituteGetter> {
    pub fn new(category: &str) -> Consumer<SubstituteGetter> {
        Consumer {
            category: category.to_string(),
            active: true,
            iterations: 0,
            get: SubstituteGetter::new(category),
        }
    }
}

impl Consumer<Category> {
    pub fn build(category: &str) -> Consumer<Category> {
        Consumer {
            category: category.to_string(),
            active: true,
            iterations: 0,
            get: Category,
        }
    }
}

impl<G: Get + Send + 'static> Consumer<G> {
    pub fn add_handler<H: messaging::Handler>(self, _handler: H) -> Self {
        self
    }

    pub fn with_settings(self, _settings: Settings) -> Self {
        self
    }

    pub fn start(self) -> ConsumerHandler<G> {
        let arc = Arc::new(Mutex::new(self));
        let thread_arc = arc.clone();

        let handle = std::thread::spawn(move || {
            loop {
                let mut consumer = thread_arc.lock().expect("the mutex to not be poisoned");

                if !consumer.deref().active {
                    break;
                }

                consumer.iterations += 1;


                let wait_time_millis = 10; //TODO: calculate via back off
                
                // Give the main thread a chance to lock the mutex
                drop(consumer);
                std::thread::sleep(std::time::Duration::from_millis(wait_time_millis));
            }
        });

        ConsumerHandler::new(arc.clone(), handle)
    }

    pub fn tick(&mut self) {
        let _messages = self.get.get(0);
    }

    pub fn get(&self) -> &G {
        &self.get
    }

    pub fn get_mut(&mut self) -> &mut G {
        &mut self.get
    }
}

pub struct ConsumerHandler<G: Get> {
    consumer: Arc<Mutex<Consumer<G>>>,
    handle: Option<JoinHandle<()>>,
}

impl<G: Get> ConsumerHandler<G> {
    pub fn new(consumer: Arc<Mutex<Consumer<G>>>, handle: JoinHandle<()>) -> Self {
        Self { consumer, handle: Some(handle) }
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
}
