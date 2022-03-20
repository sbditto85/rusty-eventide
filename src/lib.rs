use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use back_off::{constant::ConstantBackOff, BackOff};
// use controls::handler;
use messaging::{postgres::Category, *};
use position_store::{postgres::PostgresPositionStore, PositionStore, SubstitutePositionStore};
use run_time::{RunTime, SubstituteRunTime, SystemRunTime};
use settings::*;

pub mod back_off;
pub mod controls;
pub mod messaging;
pub mod position_store;
pub mod run_time;
pub mod session;
pub mod settings;

#[derive(Debug)]
pub struct Consumer<G: Get, B: BackOff, R: RunTime, P: PositionStore> {
    run_time: R,
    #[allow(dead_code)]
    category: String,
    handlers: Vec<Box<dyn Handler + Send>>,
    active: Arc<Mutex<bool>>,
    iterations: Arc<Mutex<u64>>,
    get: G,
    back_off: B,
    position: u64,
    position_update_counter: u64,
    position_store: P,
    settings: Settings,
}

impl Consumer<SubstituteGetter, ConstantBackOff, SubstituteRunTime, SubstitutePositionStore> {
    pub fn new(
        category: &str,
    ) -> Consumer<SubstituteGetter, ConstantBackOff, SubstituteRunTime, SubstitutePositionStore>
    {
        Consumer {
            run_time: SubstituteRunTime::new(),
            category: category.to_string(),
            handlers: Vec::new(),
            active: Arc::new(Mutex::new(true)),
            iterations: Arc::new(Mutex::new(0)),
            get: SubstituteGetter::new(category),
            back_off: ConstantBackOff::new(),
            position: 0,
            position_update_counter: 0,
            position_store: SubstitutePositionStore::new(),
            settings: Settings::new(),
        }
    }
}

impl Consumer<Category, ConstantBackOff, SystemRunTime, PostgresPositionStore> {
    pub fn build(
        category: &str,
    ) -> Consumer<Category, ConstantBackOff, SystemRunTime, PostgresPositionStore> {
        Consumer {
            run_time: SystemRunTime::build(),
            category: category.to_string(),
            handlers: Vec::new(),
            active: Arc::new(Mutex::new(true)),
            iterations: Arc::new(Mutex::new(0)),
            get: Category::build(category).expect("category to build"), //TODO: handle error
            back_off: ConstantBackOff::build(),
            position: 0, // TODO: have some "default?"
            position_update_counter: 0,
            position_store: PostgresPositionStore::build(),
            settings: Settings::build(),
        }
    }
}

impl<
        G: Get + Send + 'static,
        B: BackOff + Send + 'static,
        R: RunTime + Send + 'static,
        P: PositionStore + Send + 'static,
    > Consumer<G, B, R, P>
{
    pub fn add_handler<H: messaging::Handler + Send + 'static>(mut self, handler: H) -> Self {
        self.handlers.push(Box::new(handler));
        self
    }

    pub fn with_settings(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    pub fn with_back_off<B2: BackOff>(self, back_off: B2) -> Consumer<G, B2, R, P> {
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
            position: self.position,
            position_update_counter: self.position_update_counter,
            position_store: self.position_store,
            settings: self.settings,
        }
    }

    pub fn initialize(&mut self) {
        self.position = self.position_store.get();
        log::debug!("Starting at position: {}", self.position);
    }

    pub fn start(mut self) -> ConsumerHandle<G, B, R, P> {
        let active = self.active.clone();
        let iterations = self.iterations.clone();

        // TODO: Should be controlled by RunTime somehow???
        let handle = std::thread::spawn(move || -> Result<Consumer<G, B, R, P>, HandleError> {
            self.initialize();

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

    fn increment_iterations(&mut self) {
        let mut iterations = self.iterations.lock().expect("mutex to not be poisoned");
        *iterations += 1;
    }

    pub fn tick(&mut self) -> Result<u64, HandleError> {
        self.increment_iterations();

        let messages = self.get.get(self.position as i64)?; //TODO: handle position
        let messages_length = messages.len();

        for message_data in messages {
            self.handle_message(message_data)?;
        }

        Ok(messages_length as u64)
    }

    fn handle_message(&mut self, message_data: MessageData) -> Result<(), HandleError> {
        for handler in &mut self.handlers {
            handler.handle(message_data.clone())?;
        }

        self.update_position(message_data.global_position);

        Ok(())
    }

    fn update_position(&mut self, position: u64) {
        self.position = position + 1; // Set to get the next one on next fetch

        self.position_update_counter += 1;

        if self.position_update_counter >= self.settings.position_update_interval {
            self.position_store.put(position);
            self.position_update_counter = 0;
        }
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

    pub fn position_store(&self) -> &P {
        &self.position_store
    }

    pub fn position_store_mut(&mut self) -> &mut P {
        &mut self.position_store
    }
}

pub struct ConsumerHandle<G: Get, B: BackOff, R: RunTime, P: PositionStore> {
    active: Arc<Mutex<bool>>,
    iterations: Arc<Mutex<u64>>,
    handle: Option<JoinHandle<Result<Consumer<G, B, R, P>, HandleError>>>,
}

impl<G: Get, B: BackOff, R: RunTime, P: PositionStore> ConsumerHandle<G, B, R, P> {
    pub fn build(
        active: Arc<Mutex<bool>>,
        iterations: Arc<Mutex<u64>>,
        handle: JoinHandle<Result<Consumer<G, B, R, P>, HandleError>>,
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
    pub fn wait(mut self) -> Result<Consumer<G, B, R, P>, HandleError> {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("thread to join")
        } else {
            Err(HandleError::MissingHandler) //TODO: is this right?
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    use crate::position_store::PositionStoreTelemetry;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /////////////////////
    // Get
    /////////////////////

    #[test]
    fn should_ask_for_messages_every_tick() {
        init();

        // Arrange
        let mut consumer = Consumer::new("mycategory");

        // Act
        let _ = consumer.tick();

        // Assert
        let get = consumer.get();
        assert!(get.get_count() > 0);
    }

    #[test]
    fn should_return_same_number_of_queued_messages_on_tick() {
        init();

        // Arrange
        let mut consumer = Consumer::new("mycategory");

        let messages = add_messages(&mut consumer);
        let messages_count = messages.len() as u64;

        // Act
        let _ = consumer.tick();

        // Assert
        let get = consumer.get();
        assert_eq!(messages_count, get.get_messages_count());
    }

    /////////////////////
    // Running
    /////////////////////

    // Is this a good test? idk, feels a little like imperative shell to me
    #[test]
    fn should_continue_tick_until_stopped() {
        init();

        // Arrange
        // Act
        let wait_millis = 15;
        let mut consumer = Consumer::new("mycategory").start();

        // Assert
        assert!(consumer.started());
        let beginning = consumer.iterations();

        // Act
        std::thread::sleep(std::time::Duration::from_millis(wait_millis));

        consumer.stop();

        // Assert
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        assert!(
            ending > beginning,
            "Beginning: {} should be less than Ending: {}",
            beginning,
            ending
        );

        // Act
        std::thread::sleep(std::time::Duration::from_millis(wait_millis));

        // Assert
        assert_eq!(ending, consumer.iterations());
    }

    #[test]
    fn should_be_able_to_wait_until_consumer_is_done() {
        init();

        // Arrange
        let handler = controls::handler::FailingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        // Add messages so handler fails and consumer stops
        add_messages(&mut consumer);

        // Act
        let consumer = consumer.start();
        let result = consumer.wait();

        // Assert
        assert!(result.is_err());
        assert_eq!(handler.message_count(), 1);
    }

    #[test]
    fn should_stop_processing_messages_when_handler_errors_on_start() {
        init();

        // Arrange
        let handler = controls::handler::FailingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        add_messages(&mut consumer);

        // Act
        let consumer_handle = consumer.start();

        let iterations = consumer_handle.iterations.clone();

        let consumer_result = consumer_handle.wait();

        // Assert
        assert!(consumer_result.is_err());

        let actual_iterations = *iterations.lock().expect("mutex to not be poisoned");

        let expected_iterations = 1;
        let expected_message_count = 1;

        assert_eq!(
            actual_iterations, expected_iterations,
            "iterations ({}) should be {}",
            actual_iterations, expected_iterations
        );
        assert_eq!(
            handler.message_count(),
            expected_message_count,
            "message count ({}) should be {}",
            handler.message_count(),
            expected_message_count
        );
    }

    /////////////////////
    // Back off
    /////////////////////

    #[test]
    fn should_be_able_to_specify_a_back_off_strategy() {
        init();

        // Arrange
        // Choosing a small millis that still allows back off, but short test time
        let duration_millis = 8;
        let max_run_time_duration_millis = duration_millis - 2;

        let mut consumer = Consumer::new("mycategory").with_back_off(
            crate::back_off::constant::ConstantBackOff::new_with_duration(
                std::time::Duration::from_millis(duration_millis),
            ),
        );

        consumer
            .run_time_mut()
            .set_run_limit(std::time::Duration::from_millis(
                max_run_time_duration_millis,
            ));

        // Act
        let consumer_handle = consumer.start();

        let consumer = consumer_handle
            .wait()
            .expect("waiting for handler to succeed");

        // Assert
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        // Only enough time to get one iteration off due to back off being longer then max run time
        let expected_ending = 1;
        assert_eq!(expected_ending, ending);
    }

    #[test]
    fn should_be_able_to_use_last_message_count_to_determine_back_off() {
        init();

        // Arrange
        // Picking a small back off time that is still longer then the wait time
        let duration_millis = 20;
        let max_run_duration_millis = duration_millis - (duration_millis / 4); // Give a little millis buffer

        let mut consumer = Consumer::new("mycategory").with_back_off(
            crate::controls::back_off::OnNoMessageDataCount::new(std::time::Duration::from_millis(
                duration_millis,
            )),
        );

        add_messages(&mut consumer);

        consumer
            .run_time_mut()
            .set_run_limit(std::time::Duration::from_millis(max_run_duration_millis));

        // Act
        let consumer_handle = consumer.start();

        let consumer = consumer_handle.wait().expect("wait to finish successfully");

        // Assert
        assert!(consumer.stopped());

        let ending = consumer.iterations();
        // Only enough time to do one iteration with a message then immediately try for another
        //  which will cause a longer pause then the max_run_duration_millis because no messages
        let expected_ending = 2;
        assert_eq!(expected_ending, ending);
    }

    /////////////////////
    // Handler
    /////////////////////

    #[test]
    fn should_offer_messages_to_handler_on_tick() {
        init();

        // Arrange
        let handler = controls::handler::TrackingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        let messages = add_messages(&mut consumer);
        let messages_count = messages.len() as u64;

        // Act
        let _ = consumer.tick();

        // Assert
        assert_eq!(handler.message_count(), messages_count);
    }

    #[test]
    fn should_stop_processing_messages_when_handler_errors_on_tick() {
        init();

        // Arrange
        let handler = controls::handler::FailingHandler::build();
        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        add_messages(&mut consumer);

        // Act
        let _ = consumer.tick();

        // Assert
        let only_one_message_handled = 1;
        assert_eq!(handler.message_count(), only_one_message_handled);
    }

    /////////////////////
    // Position
    /////////////////////
    #[test]
    fn should_store_position_periodically_to_optimize_resume() {
        init();

        // Arrange
        let handler = controls::handler::TrackingHandler::build();
        let mut settings = Settings::new();
        settings.position_update_interval = 1;

        let mut consumer = Consumer::new("mycategory")
            .add_handler(handler.clone())
            .with_settings(settings);

        let messages = add_messages(&mut consumer);
        let messages_count = messages.len() as u64;

        // Act
        let _ = consumer.tick();

        // Assert
        let position_store = consumer.position_store();
        assert_eq!(position_store.put_count(), messages_count);
    }

    #[test]
    fn should_start_from_stored_position() {
        init();

        // Arrange
        let handler = controls::handler::TrackingHandler::build();

        let mut consumer = Consumer::new("mycategory").add_handler(handler.clone());

        let messages = add_messages(&mut consumer);
        let messages_count = messages.len() as u64;

        let position_store = consumer.position_store_mut();
        position_store.set_position(messages_count);

        // Act
        consumer.initialize();

        let _ = consumer.tick();

        // Assert
        let no_messages_processed = 0;
        assert_eq!(handler.message_count(), no_messages_processed);
    }

    #[test]
    #[ignore]
    fn should_start_at_zero_with_no_position_stored() {}

    /////////////////////
    // Helpers
    /////////////////////
    fn add_messages<
        B: BackOff + Send + 'static,
        R: RunTime + Send + 'static,
        P: PositionStore + Send + 'static,
    >(
        consumer: &mut Consumer<SubstituteGetter, B, R, P>,
    ) -> Vec<MessageData> {
        let get = consumer.get_mut();
        let messages = controls::messages::example();
        get.queue_messages(&messages);

        messages
    }
}
