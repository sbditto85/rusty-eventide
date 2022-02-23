use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use back_off::{constant::ConstantBackOff, BackOff};
// use controls::handler;
use messaging::{postgres::Category, *};
use run_time::{RunTime, SubstituteRunTime, SystemRunTime};
use settings::*;

pub mod back_off;
pub mod controls;
pub mod messaging;
pub mod run_time;
pub mod settings;

pub struct Consumer<G: Get, B: BackOff, R: RunTime> {
    run_time: R,
    #[allow(dead_code)]
    category: String,
    handlers: Vec<Box<dyn Handler + Send>>,
    active: Arc<Mutex<bool>>,
    iterations: Arc<Mutex<u64>>,
    get: G,
    back_off: B,
}

impl Consumer<SubstituteGetter, ConstantBackOff, SubstituteRunTime> {
    pub fn new(category: &str) -> Consumer<SubstituteGetter, ConstantBackOff, SubstituteRunTime> {
        Consumer {
            run_time: SubstituteRunTime::new(),
            category: category.to_string(),
            handlers: Vec::new(),
            active: Arc::new(Mutex::new(true)),
            iterations: Arc::new(Mutex::new(0)),
            get: SubstituteGetter::new(category),
            back_off: ConstantBackOff::new(),
        }
    }
}

impl Consumer<Category, ConstantBackOff, SystemRunTime> {
    pub fn build(category: &str) -> Consumer<Category, ConstantBackOff, SystemRunTime> {
        Consumer {
            run_time: SystemRunTime::build(),
            category: category.to_string(),
            handlers: Vec::new(),
            active: Arc::new(Mutex::new(true)),
            iterations: Arc::new(Mutex::new(0)),
            get: Category,
            back_off: ConstantBackOff::build(),
        }
    }
}

impl<G: Get + Send + 'static, B: BackOff + Send + 'static, R: RunTime + Send + 'static>
    Consumer<G, B, R>
{
    pub fn add_handler<H: messaging::Handler + Send + 'static>(mut self, handler: H) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    pub fn with_settings(self, _settings: Settings) -> Self {
        self
    }

    pub fn with_back_off<B2: BackOff>(self, back_off: B2) -> Consumer<G, B2, R> {
        // Is there a better way to do this? where I only have to specify back_off?
        // can't use `..self` because B and B2 are different types :(
        Consumer {
            run_time: self.run_time,
            category: self.category,
            handlers: self.handlers,
            active: self.active,
            iterations: self.iterations,
            get: self.get,
            back_off,
        }
    }

    pub fn start(mut self) -> ConsumerHandle<G, B, R> {
        let active = self.active.clone();
        let iterations = self.iterations.clone();

        let handle = std::thread::spawn(move || -> Result<Consumer<G, B, R>, HandleError> {
            let mut should_continue = true;
            while should_continue {
                let active = self.active.lock().expect("mutex to not be poisoned");

                if !active.deref() {
                    break;
                }

                // Give the main thread a chance to lock the mutex
                drop(active);

                let iteration_message_count = self.tick().map_err(|error| {
                    self.set_inactive();
                    error
                })?;

                let wait_time = self.back_off.duration(iteration_message_count);

                self.run_time.sleep(wait_time);
                should_continue = self.run_time.should_continue();
            }

            self.set_inactive();
            Ok(self)
        });

        ConsumerHandle::build(active, iterations, handle)
    }

    fn set_inactive(&mut self) {
        let mut active = self.active.lock().expect("mutex to not be poisoned");
        *active = false;
    }

    pub fn stopped(&self) -> bool {
        !*self.active.lock().expect("mutex to not be poisoned")
    }

    pub fn iterations(&self) -> u64 {
        *self.iterations.lock().expect("mutex to not be poisoned")
    }

    pub fn tick(&mut self) -> Result<u64, HandleError> {
        let mut iterations = self.iterations.lock().expect("mutex to not be poisoned");
        *iterations += 1;
        let messages = self.get.get(0); //TODO: handle position
        let messages_length = messages.len();

        for message in messages {
            for handler in &mut self.handlers {
                handler.handle(message.clone())?;
            }
        }

        Ok(messages_length as u64)
    }

    pub fn get(&self) -> &G {
        &self.get
    }

    pub fn get_mut(&mut self) -> &mut G {
        &mut self.get
    }

    pub fn run_time_mut(&mut self) -> &mut R {
        &mut self.run_time
    }
}

pub struct ConsumerHandle<G: Get, B: BackOff, R: RunTime> {
    active: Arc<Mutex<bool>>,
    iterations: Arc<Mutex<u64>>,
    handle: Option<JoinHandle<Result<Consumer<G, B, R>, HandleError>>>,
}

impl<G: Get, B: BackOff, R: RunTime> ConsumerHandle<G, B, R> {
    pub fn build(
        active: Arc<Mutex<bool>>,
        iterations: Arc<Mutex<u64>>,
        handle: JoinHandle<Result<Consumer<G, B, R>, HandleError>>,
    ) -> Self {
        Self {
            active,
            iterations,
            handle: Some(handle),
        }
    }

    pub fn iterations(&self) -> u64 {
        *self.iterations.lock().expect("mutex to not be poisoned")
    }

    pub fn stop(&mut self) {
        let mut active = self.active.lock().expect("mutex to not be poisoned");
        *active = false;
        // Allow runner to get mutex
        drop(active);

        self.handle.take().map(|thread| thread.join());
    }

    pub fn started(&self) -> bool {
        *self.active.lock().expect("mutex to not be poisoned")
    }

    pub fn stopped(&self) -> bool {
        !*self.active.lock().expect("mutex to not be poisoned")
    }

    /// Will run until completion if you need to run again start a new consumer
    pub fn wait(mut self) -> Result<Consumer<G, B, R>, HandleError> {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("thread to join")
        } else {
            Err(HandleError::MissingHandler) //TODO: is this right?
        }
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

        let _ = consumer.tick();

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

        let _ = consumer.tick();

        let get = consumer.get();

        assert_eq!(messages_count, get.get_messages_count());
    }

    /////////////////////
    // Running
    /////////////////////

    // Is this a good test? idk, feels a little like imperative shell to me
    #[test]
    fn should_continue_tick_until_stopped() {
        let wait_millis = 15;
        let mut consumer = Consumer::new("mycategory").start();

        assert!(consumer.started());
        let beginning = consumer.iterations();

        std::thread::sleep(std::time::Duration::from_millis(wait_millis));

        consumer.stop();
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        assert!(
            ending > beginning,
            "Beginning: {} should be less than Ending: {}",
            beginning,
            ending
        );

        std::thread::sleep(std::time::Duration::from_millis(wait_millis));

        assert_eq!(ending, consumer.iterations());
    }

    #[test]
    fn should_be_able_to_wait_until_consumer_is_done() {
        let handler = controls::handler::FailingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        // Add messages so handler fails and consumer stops
        let get = consumer.get_mut();
        let messages = controls::messages::example();
        get.queue_messages(&messages);

        let consumer = consumer.start();

        let result = consumer.wait();

        assert!(result.is_err());
        assert_eq!(handler.message_count(), 1);
    }

    #[test]
    #[ignore]
    fn should_stop_processing_messages_when_handler_errors_on_start() {
        let handler = controls::handler::FailingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        let get = consumer.get_mut();
        let messages = controls::messages::example();
        get.queue_messages(&messages);

        let back_off = consumer.back_off.duration(1);

        let mut consumer = consumer.start();

        assert!(consumer.started());

        std::thread::sleep(back_off);

        consumer.stop();
        assert!(consumer.stopped());

        assert_eq!(consumer.iterations(), 1);
        assert_eq!(handler.message_count(), 1);
    }

    /////////////////////
    // Back off
    /////////////////////

    #[test]
    fn should_be_able_to_specify_a_back_off_strategy() {
        // Choosing a small millis that still allows back off, but short test time
        let duration = 8;
        let thread_sleep_duration = duration - 2;

        let mut consumer = Consumer::new("mycategory").with_back_off(
            crate::back_off::constant::ConstantBackOff::new_with_duration(
                std::time::Duration::from_millis(duration),
            ),
        );

        consumer
            .run_time_mut()
            .set_run_limit(std::time::Duration::from_millis(thread_sleep_duration));

        let consumer_handle = consumer.start();

        assert!(consumer_handle.started());
        let beginning = consumer_handle.iterations();

        let consumer = consumer_handle
            .wait()
            .expect("waiting for handler to succeed");
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        // Only enough time to get one iteration off due to back off being longer then test sleep
        let expected_ending = beginning + 1;
        assert_eq!(expected_ending, ending);
    }

    #[test]
    fn should_be_able_to_use_last_message_count_to_determine_back_off() {
        // Picking a small back off time that is still longer then the wait time
        let duration_millis = 20;
        let max_run_duration_millis = duration_millis - (duration_millis / 4); // Give a little millis buffer

        let mut consumer = Consumer::new("mycategory").with_back_off(
            crate::controls::back_off::OnNoMessageCount::new(std::time::Duration::from_millis(
                duration_millis,
            )),
        );

        let get = consumer.get_mut();
        let messages = controls::messages::example();
        get.queue_messages(&messages);

        consumer
            .run_time_mut()
            .set_run_limit(std::time::Duration::from_millis(max_run_duration_millis));

        let consumer_handle = consumer.start();

        std::thread::sleep(std::time::Duration::from_millis(10)); // Allow consumer thread to start

        assert!(consumer_handle.started());
        let beginning = consumer_handle.iterations();

        let consumer = consumer_handle.wait().expect("wait to finish successfully");
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

        let _ = consumer.tick();

        assert_eq!(handler.message_count(), messages_count);
    }

    #[test]
    fn should_stop_processing_messages_when_handler_errors_on_tick() {
        let handler = controls::handler::FailingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        let get = consumer.get_mut();
        let messages = controls::messages::example();
        get.queue_messages(&messages);

        let _ = consumer.tick();

        let only_one_message_handled = 1;
        assert_eq!(handler.message_count(), only_one_message_handled);
    }

    /////////////////////
    // Position
    /////////////////////
    #[test]
    #[ignore]
    fn should_store_position_periodically_to_optimize_resume() {}
}
